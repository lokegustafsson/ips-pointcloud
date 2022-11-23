use std::{
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

pub fn solve_scan(xyzi: &[(f32, f32, f32, u16)]) -> Vec<(u16, u16)> {
    let n = xyzi.len();
    assert!(n <= u16::MAX as usize);

    let mut xyzi = xyzi.to_owned();
    assert_eq!(n, xyzi.len());

    xyzi.sort_unstable_by(|(ax, _, _, _), (bx, _, _, _)| ax.total_cmp(bx));

    let mut first_relevant = 0;
    let mut ans = Vec::new();
    for i in 0..n {
        for j in first_relevant..i {
            let (xi, yi, zi, ii) = xyzi[i];
            let (xj, yj, zj, ij) = xyzi[j];
            let dx = xi - xj;
            if dx > THRESHOLD {
                first_relevant += 1;
            } else {
                let dy = yi - yj;
                let dz = zi - zj;
                if dx * dx + dy * dy + dz * dz < THRESHOLD2 {
                    ans.push(if ii < ij { (ii, ij) } else { (ij, ii) });
                }
            }
        }
    }
    ans
}
pub fn solve_subscan(xyzi: &[(f32, f32, f32, u16)]) -> Vec<(u16, u16)> {
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

    let mut xyzi = xyzi.to_owned();
    assert_eq!(n, xyzi.len());

    xyzi.sort_unstable_by(|(ax, _, _, _), (bx, _, _, _)| ax.total_cmp(bx));

    let mut slice_queue: VecDeque<PointY> = VecDeque::new();
    let mut slice_set: BTreeSet<PointY> = BTreeSet::new();
    let mut ans = Vec::new();
    for i in 0..n {
        let (xi, yi, zi, ii) = xyzi[i];
        while slice_queue.front().is_some() && xi - slice_queue.front().unwrap().x > THRESHOLD {
            slice_set.remove(&slice_queue.pop_front().unwrap());
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
                ans.push(if ii < *ij { (ii, *ij) } else { (*ij, ii) });
            }
        }
        slice_set.insert(slice_queue.back().unwrap().clone());
    }
    ans
}
// TODO
// 2. Avoid realloc on ans reducing
// 3. Input data as AoS rather than SoA
pub fn solve_subscan_threaded(
    xyzi: &mut [(f32, f32, f32, u16)],
    parallel: NonZeroUsize,
    ret: &mut Vec<MaybeUninit<(u16, u16)>>,
) {
    use rayon::prelude::{IntoParallelIterator, ParallelIterator, ParallelSliceMut};
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

    assert_eq!(n, xyzi.len());
    xyzi.par_sort_unstable_by(|(ax, _, _, _), (bx, _, _, _)| ax.total_cmp(bx));

    fn solve_interval(xyzi: &[(f32, f32, f32, u16)], start: usize, end: usize) -> Vec<(u16, u16)> {
        let n = xyzi.len();
        assert!(n <= (u16::MAX - 10) as usize);

        let pre_start_x = xyzi[start].0 - THRESHOLD;

        let mut slice_queue: VecDeque<PointY> = VecDeque::new();
        let mut slice_set: BTreeSet<PointY> = BTreeSet::new();
        let mut ans = Vec::new();
        for i in {
            let (Ok(pre_start) | Err(pre_start)) =
                xyzi.binary_search_by(|&(x, _, _, _)| x.total_cmp(&pre_start_x));
            pre_start
        }..start
        {
            let (x, y, z, idx) = xyzi[i];
            slice_queue.push_back(PointY { x, y, z, idx });
            slice_set.insert(slice_queue.back().unwrap().clone());
        }
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
                    ans.push(if ii < *ij { (ii, *ij) } else { (*ij, ii) });
                }
            }
            slice_set.insert(slice_queue.back().unwrap().clone());
        }
        ans
    }
    if false {
        let chunks: Vec<Vec<(u16, u16)>> = (0..parallel.get())
            .into_par_iter()
            .map(|chunk_idx| {
                let chunk_size = (xyzi.len() / parallel.get()).max(1).min(xyzi.len());
                let start = chunk_idx * chunk_size;
                let end = if chunk_idx + 1 == parallel.get() {
                    xyzi.len()
                } else {
                    usize::min(start + chunk_size, xyzi.len())
                };
                if start < end {
                    solve_interval(&xyzi, start, end)
                } else {
                    Vec::new()
                }
            })
            .collect();
        let total_ans_length = chunks.iter().map(|chunk| chunk.len()).sum();
        ret.truncate(0);
        ret.reserve(total_ans_length);
        ret.resize(total_ans_length, MaybeUninit::uninit());

        let mut chunk_assignments = Vec::new();
        let mut ret_suffix: &mut [MaybeUninit<(u16, u16)>] = ret.as_mut();
        for chunk in chunks {
            let (ret_chunk, new_ret_suffix) = ret_suffix.split_at_mut(chunk.len());
            ret_suffix = new_ret_suffix;
            chunk_assignments.push((ret_chunk, chunk));
        }
        assert_eq!(ret_suffix.len(), 0);
        chunk_assignments
            .into_par_iter()
            .for_each(|(ret_chunk, chunk)| {
                assert_eq!(ret_chunk.len(), chunk.len());
                ret_chunk.copy_from_slice(&slice_wrap_maybeinit(&chunk))
            });
    } else {
        *ret = (0..parallel.get())
            .into_par_iter()
            .map(|chunk_idx| {
                let chunk_size = (xyzi.len() / parallel.get()).max(1).min(xyzi.len());
                let start = chunk_idx * chunk_size;
                let end = if chunk_idx + 1 == parallel.get() {
                    xyzi.len()
                } else {
                    usize::min(start + chunk_size, xyzi.len())
                };
                if start < end {
                    vec_wrap_maybeinit(solve_interval(&xyzi, start, end))
                } else {
                    Vec::new()
                }
            })
            .reduce(Vec::new, |mut a, b| {
                if a.is_empty() {
                    b
                } else {
                    a.extend_from_slice(&b);
                    a
                }
            })
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
