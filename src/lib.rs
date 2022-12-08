use rayon::prelude::{
    IndexedParallelIterator, IntoParallelIterator, ParallelIterator, ParallelSliceMut,
};
use std::{
    cell::UnsafeCell,
    cmp::Ordering,
    collections::BTreeSet,
    io::Read,
    mem::{self, MaybeUninit},
    num::NonZeroUsize,
};

const THRESHOLD: f32 = 0.05;
const THRESHOLD2: f32 = 0.05 * 0.05;

pub fn solve_naive(xyzi: &[(f32, f32, f32, u16)]) -> Vec<(u16, u16)> {
    assert!(xyzi.len() <= u16::MAX as usize);

    let mut ans = Vec::new();
    for i in 0..xyzi.len() {
        for j in 0..i {
            let dx = xyzi[i].0 - xyzi[j].0;
            let dy = xyzi[i].1 - xyzi[j].1;
            let dz = xyzi[i].2 - xyzi[j].2;
            if dx * dx + dy * dy + dz * dz < THRESHOLD2 {
                ans.push((xyzi[j].3, xyzi[i].3));
            }
        }
    }
    ans
}

pub trait IntervalSolver {
    const USE_SOA: bool;
    fn solve_interval(
        xyzi: &[(f32, f32, f32, u16)],
        xyzi_soa: (&[f32], &[f32], &[f32], &[u16]),
        start: usize,
        end: usize,
        ret: &mut Vec<(u16, u16)>,
    );
}
pub fn solve_threaded<IS: IntervalSolver>(
    xyzi: &mut [(f32, f32, f32, u16)],
    (x_soa, y_soa, z_soa, i_soa): (&mut Vec<f32>, &mut Vec<f32>, &mut Vec<f32>, &mut Vec<u16>),
    parallel: NonZeroUsize,
    ret: &mut UnsafeCell<Vec<MaybeUninit<(u16, u16)>>>,
) {
    let n = xyzi.len();
    assert!(n <= (u16::MAX - 10) as usize);

    assert_eq!(n, xyzi.len());
    xyzi.par_sort_unstable_by(|(ax, _, _, _), (bx, _, _, _)| ax.total_cmp(bx));
    fn soa(
        xyzi: &[(f32, f32, f32, u16)],
        (x_soa, y_soa, z_soa, i_soa): (&mut Vec<f32>, &mut Vec<f32>, &mut Vec<f32>, &mut Vec<u16>),
    ) {
        [Ok(x_soa), Ok(y_soa), Ok(z_soa), Err(i_soa)]
            .into_par_iter()
            .enumerate()
            .for_each(|(i, dst)| match (i, dst) {
                (0, Ok(x)) => x.extend(xyzi.iter().map(|xyzi| xyzi.0)),
                (1, Ok(y)) => y.extend(xyzi.iter().map(|xyzi| xyzi.1)),
                (2, Ok(z)) => z.extend(xyzi.iter().map(|xyzi| xyzi.2)),
                (3, Err(i)) => i.extend(xyzi.iter().map(|xyzi| xyzi.3)),
                _ => unreachable!(),
            });
    }
    if IS::USE_SOA {
        x_soa.truncate(0);
        y_soa.truncate(0);
        z_soa.truncate(0);
        i_soa.truncate(0);
        soa(&xyzi, (x_soa, y_soa, z_soa, i_soa));
        assert_eq!(n, x_soa.len());
        assert_eq!(n, y_soa.len());
        assert_eq!(n, z_soa.len());
        assert_eq!(n, i_soa.len());
    }

    #[derive(Clone, Copy)]
    struct RetPtr<'a>(&'a UnsafeCell<Vec<MaybeUninit<(u16, u16)>>>);
    impl<'a> RetPtr<'a> {
        unsafe fn get_mut(self) -> &'a mut Vec<MaybeUninit<(u16, u16)>> {
            &mut *UnsafeCell::raw_get(self.0)
        }
    }
    unsafe impl<'a> Send for RetPtr<'a> {}
    unsafe impl<'a> Sync for RetPtr<'a> {}

    ret.get_mut().truncate(0);
    let ret_ref = RetPtr(ret);

    (0..parallel.get())
        .into_par_iter()
        .map(|chunk_idx| {
            let chunk_size = (xyzi.len() / parallel.get()).clamp(1, xyzi.len());
            let start = chunk_idx * chunk_size;
            let end = if chunk_idx + 1 == parallel.get() {
                xyzi.len()
            } else {
                usize::min(start + chunk_size, xyzi.len())
            };
            let mut ret = Vec::new();
            if start < end {
                IS::solve_interval(xyzi, (&x_soa, &y_soa, &z_soa, &i_soa), start, end, &mut ret);
            }
            (vec_wrap_maybeinit(ret), chunk_idx == 0)
        })
        .reduce(
            || (Vec::new(), false),
            move |(mut a, a_first), (b, b_first)| {
                if a_first || b_first {
                    {
                        // SAFETY: `a_first || b_first` only holds in single branch of reduction tree.
                        // We are hence the only thread currently executing this block, since we
                        // must return before another thread can have `a_first || b_first`.
                        let ret_inner = unsafe { ret_ref.get_mut() };
                        ret_inner.extend_from_slice(&a);
                        ret_inner.extend_from_slice(&b);
                    }
                    (Vec::new(), true)
                } else if a.is_empty() {
                    (b, false)
                } else {
                    a.extend_from_slice(&b);
                    (a, false)
                }
            },
        );
}
pub struct ScanSolver;
impl IntervalSolver for ScanSolver {
    const USE_SOA: bool = true;
    fn solve_interval(
        _xyzi: &[(f32, f32, f32, u16)],
        (vx, vy, vz, vi): (&[f32], &[f32], &[f32], &[u16]),
        start: usize,
        end: usize,
        ret: &mut Vec<(u16, u16)>,
    ) {
        ret.truncate(0);

        let mut first_relevant = {
            let pre_start_x = vx[start] - THRESHOLD;
            let (Ok(i) | Err(i)) = vx.binary_search_by(|&x| x.total_cmp(&pre_start_x));
            i
        };
        for i in start..end {
            let (xi, yi, zi, ii) = (vx[i], vy[i], vz[i], vi[i]);
            while xi - vx[first_relevant] > THRESHOLD {
                first_relevant += 1;
            }
            let end = i - (i - first_relevant) % 8;
            for j in (first_relevant..end).step_by(8) {
                unsafe { simd((vx, vy, vz, vi), (xi, yi, zi, ii), j, ret) }
            }
            for j in end..i {
                let (xj, yj, zj, ij) = (vx[j], vy[j], vz[j], vi[j]);
                let dx = xi - xj;
                let dy = yi - yj;
                let dz = zi - zj;
                if dx * dx + dy * dy + dz * dz < THRESHOLD2 {
                    ret.push(if ii < ij { (ii, ij) } else { (ij, ii) });
                }
            }
        }
        unsafe fn simd(
            (vx, vy, vz, vi): (&[f32], &[f32], &[f32], &[u16]),
            (xi, yi, zi, ii): (f32, f32, f32, u16),
            j: usize,
            ret: &mut Vec<(u16, u16)>,
        ) {
            use std::arch::x86_64::{
                _mm256_cmp_ps, _mm256_fmadd_ps, _mm256_loadu_ps, _mm256_mul_ps, _mm256_set1_ps,
                _mm256_storeu_ps, _mm256_sub_ps, _mm256_testz_ps, _CMP_LT_OQ,
            };
            let xi = _mm256_set1_ps(xi);
            let yi = _mm256_set1_ps(yi);
            let zi = _mm256_set1_ps(zi);
            let threshold2 = _mm256_set1_ps(THRESHOLD2);
            let xj = _mm256_loadu_ps(&vx[j] as *const f32);
            let yj = _mm256_loadu_ps(&vy[j] as *const f32);
            let zj = _mm256_loadu_ps(&vz[j] as *const f32);
            let dx = _mm256_sub_ps(xi, xj);
            let dy = _mm256_sub_ps(yi, yj);
            let dz = _mm256_sub_ps(zi, zj);
            let delta2 = _mm256_fmadd_ps(dz, dz, _mm256_fmadd_ps(dy, dy, _mm256_mul_ps(dx, dx)));
            let mask = _mm256_cmp_ps::<_CMP_LT_OQ>(delta2, threshold2);
            let allzero = _mm256_testz_ps(mask, mask) == 1;
            if allzero {
                return;
            }
            let mut mask_f32 = [0f32; 8];
            _mm256_storeu_ps(&mut mask_f32 as *mut f32, mask);
            let mut ans_chunk: [(u16, u16); 8] = [(0, 0); 8];
            assert!(j + 8 <= vi.len());
            for t in 0..8 {
                if mask_f32[t].to_bits() == 0 {
                    ans_chunk[t] = (0, 0)
                } else {
                    let ij = vi[j + t];
                    if ii < ij {
                        ans_chunk[t] = (ii, ij)
                    } else {
                        ans_chunk[t] = (ij, ii)
                    }
                }
            }
            for t in 0..8 {
                match ans_chunk[t] {
                    (0, 0) => {}
                    (a, b) => ret.push((a, b)),
                }
            }
        }
    }
}
pub struct SubscanSolver;
impl IntervalSolver for SubscanSolver {
    const USE_SOA: bool = false;
    fn solve_interval(
        xyzi: &[(f32, f32, f32, u16)],
        _xyzi_soa: (&[f32], &[f32], &[f32], &[u16]),
        start: usize,
        end: usize,
        ret: &mut Vec<(u16, u16)>,
    ) {
        #[derive(Clone)]
        struct PointY {
            y: f32,
            x: f32,
            z: f32,
            idx: u16,
        }
        impl PartialEq for PointY {
            fn eq(&self, other: &Self) -> bool {
                self.idx == other.idx
            }
        }
        impl PartialOrd for PointY {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(f32::total_cmp(&self.y, &other.y).then_with(|| self.idx.cmp(&other.idx)))
            }
        }
        impl Eq for PointY {}
        impl Ord for PointY {
            fn cmp(&self, other: &Self) -> Ordering {
                f32::total_cmp(&self.y, &other.y).then_with(|| self.idx.cmp(&other.idx))
            }
        }
        let n = xyzi.len();
        assert!(n <= (u16::MAX - 10) as usize);
        assert!(start <= end);
        assert!(end <= n);

        ret.truncate(0);

        let pre_start_x = xyzi[start].0 - THRESHOLD;
        let (Ok(pre_start) | Err(pre_start)) =
            xyzi.binary_search_by(|&(x, _, _, _)| x.total_cmp(&pre_start_x));

        let mut first_index = pre_start;

        let mut slice_set: BTreeSet<PointY> = (pre_start..start)
            .map(|i| {
                let (x, y, z, idx) = xyzi[i];
                PointY { x, y, z, idx }
            })
            .collect();

        for i in start..end {
            let (xi, yi, zi, ii) = xyzi[i];
            while first_index < i && xi - xyzi[first_index].0 > THRESHOLD {
                let (x, y, z, idx) = xyzi[first_index];
                slice_set.remove(&PointY { y, x, z, idx });
                first_index += 1;
            }
            for &PointY {
                y: yj,
                x: xj,
                z: zj,
                idx: ij,
            } in slice_set.range(
                PointY {
                    x: 0.0,
                    y: yi - THRESHOLD,
                    z: 0.0,
                    idx: u16::MAX - 1,
                }..PointY {
                    x: 0.0,
                    y: yi + THRESHOLD,
                    z: 0.0,
                    idx: u16::MAX,
                },
            ) {
                let dx = xi - xj;
                let dy = yi - yj;
                let dz = zi - zj;
                if dx * dx + dy * dy + dz * dz < THRESHOLD2 {
                    ret.push(if ii < ij { (ii, ij) } else { (ij, ii) });
                }
            }
            slice_set.insert(PointY {
                x: xi,
                y: yi,
                z: zi,
                idx: ii,
            });
        }
    }
}

pub fn parse_input(mut source: impl Read) -> Vec<(f32, f32, f32, u16)> {
    let mut input = String::new();
    source.read_to_string(&mut input).unwrap();
    let mut ret = Vec::new();
    for (i, line) in input.lines().enumerate() {
        let mut nums = line.split(' ');
        let x = nums.next().unwrap().parse().unwrap();
        let y = nums.next().unwrap().parse().unwrap();
        let z = nums.next().unwrap().parse().unwrap();
        ret.push((x, y, z, i as u16));
        assert_eq!(nums.next(), None);
    }
    ret
}
pub fn compute_closeness(xyzi: &[(f32, f32, f32, u16)]) -> [usize; 3] {
    return vec![
        Box::new(|| closeness_1d(&xyzi.iter().map(|(x, _, _, _)| *x).collect::<Vec<_>>()))
            as Box<dyn Sync + Fn() -> usize>,
        Box::new(|| closeness_1d(&xyzi.iter().map(|(_, y, _, _)| *y).collect::<Vec<_>>())),
        Box::new(|| closeness_1d(&xyzi.iter().map(|(_, _, z, _)| *z).collect::<Vec<_>>())),
    ]
    .into_par_iter()
    .map(|fun| fun())
    .collect::<Vec<usize>>()
    .try_into()
    .unwrap();

    fn closeness_1d(x: &[f32]) -> usize {
        let mut x = Vec::from(x);
        x.par_sort_unstable_by(|a, b| a.total_cmp(b));
        let mut start = 0;
        let mut ans = 0;
        for end in 1..x.len() {
            while x[end] - x[start] > THRESHOLD {
                start += 1;
            }
            ans += end - start;
        }
        ans
    }
}

/// # Safety
///
/// Input array must be fully initialized
pub unsafe fn slice_assume_init(s: &mut [MaybeUninit<(u16, u16)>]) -> &mut [(u16, u16)] {
    mem::transmute(s)
}
pub fn vec_wrap_maybeinit(s: Vec<(u16, u16)>) -> Vec<MaybeUninit<(u16, u16)>> {
    unsafe { mem::transmute(s) }
}
