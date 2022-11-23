use std::{
    cell::UnsafeCell,
    cmp::Ordering,
    collections::{BTreeSet, VecDeque},
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
    fn solve_interval(
        xyzi: &[(f32, f32, f32, u16)],
        start: usize,
        end: usize,
        ret: &mut Vec<(u16, u16)>,
    );
}
pub fn solve_threaded<IS: IntervalSolver>(
    xyzi: &mut [(f32, f32, f32, u16)],
    parallel: NonZeroUsize,
    ret: &mut Vec<MaybeUninit<(u16, u16)>>,
) {
    use rayon::prelude::{IntoParallelIterator, ParallelIterator, ParallelSliceMut};

    let n = xyzi.len();
    assert!(n <= (u16::MAX - 10) as usize);

    assert_eq!(n, xyzi.len());
    xyzi.par_sort_unstable_by(|(ax, _, _, _), (bx, _, _, _)| ax.total_cmp(bx));

    #[derive(Clone, Copy)]
    struct RetPtr(*const UnsafeCell<Vec<MaybeUninit<(u16, u16)>>>);
    impl RetPtr {
        unsafe fn get_mut(self) -> &'static mut Vec<MaybeUninit<(u16, u16)>> {
            &mut *UnsafeCell::raw_get(self.0)
        }
    }
    unsafe impl Send for RetPtr {}
    unsafe impl Sync for RetPtr {}
    let ret_ptr = unsafe {
        RetPtr(mem::transmute::<*mut Vec<_>, *const UnsafeCell<Vec<_>>>(
            ret,
        ))
    };
    ret.truncate(0);

    (0..parallel.get())
        .into_par_iter()
        .map(|chunk_idx| {
            let chunk_size = (xyzi.len() / parallel.get()).max(1).min(xyzi.len());
            let start = chunk_idx * chunk_size;
            let end = if chunk_idx + 1 == parallel.get() {
                xyzi.len()
            } else {
                usize::min(start + chunk_size, xyzi.len())
            };
            let mut ret = Vec::new();
            if start < end {
                IS::solve_interval(&xyzi, start, end, &mut ret);
            }
            (vec_wrap_maybeinit(ret), chunk_idx == 0)
        })
        .reduce(
            || (Vec::new(), false),
            move |(mut a, a_first), (b, b_first)| {
                if a_first || b_first {
                    {
                        let ret_inner = unsafe { ret_ptr.get_mut() };
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
    fn solve_interval(
        xyzi: &[(f32, f32, f32, u16)],
        start: usize,
        end: usize,
        ret: &mut Vec<(u16, u16)>,
    ) {
        ret.truncate(0);

        let mut first_relevant = {
            let pre_start_x = xyzi[start].0 - THRESHOLD;
            let (Ok(i) | Err(i)) = xyzi.binary_search_by(|&(x, _, _, _)| x.total_cmp(&pre_start_x));
            i
        };
        for i in start..end {
            let (xi, yi, zi, ii) = xyzi[i];
            for j in first_relevant..i {
                let (xj, yj, zj, ij) = xyzi[j];
                let dx = xi - xj;
                if dx > THRESHOLD {
                    first_relevant += 1;
                } else {
                    let dy = yi - yj;
                    let dz = zi - zj;
                    if dx * dx + dy * dy + dz * dz < THRESHOLD2 {
                        ret.push(if ii < ij { (ii, ij) } else { (ij, ii) });
                    }
                }
            }
        }
    }
}
pub struct SubscanSolver;
impl IntervalSolver for SubscanSolver {
    fn solve_interval(
        xyzi: &[(f32, f32, f32, u16)],
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

        ret.truncate(0);

        let pre_start_x = xyzi[start].0 - THRESHOLD;
        let (Ok(pre_start) | Err(pre_start)) =
            xyzi.binary_search_by(|&(x, _, _, _)| x.total_cmp(&pre_start_x));

        let mut slice_queue: VecDeque<PointY> = (pre_start..start)
            .map(|i| {
                let (x, y, z, idx) = xyzi[i];
                PointY { x, y, z, idx }
            })
            .collect();
        let mut slice_set: BTreeSet<PointY> = slice_queue.iter().cloned().collect();

        for i in start..end {
            let (xi, yi, zi, ii) = xyzi[i];
            while slice_queue.front().is_some() && xi - slice_queue.front().unwrap().x > THRESHOLD {
                assert!(slice_set.remove(&slice_queue.pop_front().unwrap()));
            }
            slice_queue.push_back(PointY {
                x: xi,
                y: yi,
                z: zi,
                idx: ii,
            });
            for PointY {
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
                    ret.push(if ii < *ij { (ii, *ij) } else { (*ij, ii) });
                }
            }
            slice_set.insert(slice_queue.back().unwrap().clone());
        }
    }
}

pub fn parse_input(mut source: impl Read) -> Vec<(f32, f32, f32, u16)> {
    let mut input = String::new();
    source.read_to_string(&mut input).unwrap();
    let mut ret = Vec::new();
    for (i, line) in input.lines().enumerate() {
        let mut nums = line.split(" ");
        let x = nums.next().unwrap().parse().unwrap();
        let y = nums.next().unwrap().parse().unwrap();
        let z = nums.next().unwrap().parse().unwrap();
        ret.push((x, y, z, i as u16));
        assert_eq!(nums.next(), None);
    }
    ret
}
pub fn compute_closeness(xyzi: &[(f32, f32, f32, u16)]) -> [usize; 3] {
    use rayon::prelude::{IntoParallelIterator, ParallelIterator, ParallelSliceMut};
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

pub unsafe fn slice_assume_init(s: &mut [MaybeUninit<(u16, u16)>]) -> &mut [(u16, u16)] {
    mem::transmute(s)
}
pub fn vec_wrap_maybeinit(s: Vec<(u16, u16)>) -> Vec<MaybeUninit<(u16, u16)>> {
    unsafe { mem::transmute(s) }
}
fn slice_wrap_maybeinit(s: &[(u16, u16)]) -> &[MaybeUninit<(u16, u16)>] {
    unsafe { mem::transmute(s) }
}
