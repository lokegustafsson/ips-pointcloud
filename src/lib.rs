use std::{
    cmp::Ordering,
    collections::{BTreeSet, VecDeque},
    io::Read,
};

const THRESHOLD: f32 = 0.05;
const THRESHOLD2: f32 = 0.05 * 0.05;

pub fn solve_naive([x, y, z]: [&[f32]; 3]) -> Vec<(u16, u16)> {
    let n = x.len();
    assert_eq!(n, x.len());
    assert_eq!(n, y.len());
    assert_eq!(n, z.len());
    assert!(x.len() <= u16::MAX as usize);

    let mut ans = Vec::new();
    for i in 0..x.len() {
        for j in 0..i {
            let dx = x[i] - x[j];
            let dy = y[i] - y[j];
            let dz = z[i] - z[j];
            if dx * dx + dy * dy + dz * dz < THRESHOLD2 {
                ans.push((j as u16, i as u16));
            }
        }
    }
    ans
}
pub fn solve_scan([x, y, z]: [&[f32]; 3]) -> Vec<(u16, u16)> {
    let n = x.len();
    assert_eq!(n, x.len());
    assert_eq!(n, y.len());
    assert_eq!(n, z.len());
    assert!(n <= u16::MAX as usize);

    let mut xyzi: Vec<(f32, f32, f32, u16)> =
        (0..x.len()).map(|i| (x[i], y[i], z[i], i as u16)).collect();
    assert_eq!(n, xyzi.len());
    xyzi.sort_unstable_by(|(ax, _, _, _), (bx, _, _, _)| ax.total_cmp(bx));
    let mut x = vec![0.0; n];
    let mut y = vec![0.0; n];
    let mut z = vec![0.0; n];
    let mut idx = vec![0; n];
    for i in 0..n {
        (x[i], y[i], z[i], idx[i]) = xyzi[i];
    }
    let mut first_relevant = 0;
    let mut ans = Vec::new();
    for i in 0..n {
        while x[i] - x[first_relevant] > THRESHOLD {
            first_relevant += 1;
        }
        for j in first_relevant..i {
            let dx = x[i] - x[j];
            let dy = y[j] - y[i];
            let dz = z[j] - z[i];
            if dx * dx + dy * dy + dz * dz < THRESHOLD2 {
                ans.push((u16::min(idx[i], idx[j]), u16::max(idx[i], idx[j])));
            }
        }
    }
    ans
}

pub fn solve_scan_aos([x, y, z]: [&[f32]; 3]) -> Vec<(u16, u16)> {
    let n = x.len();
    assert_eq!(n, x.len());
    assert_eq!(n, y.len());
    assert_eq!(n, z.len());
    assert!(n <= u16::MAX as usize);

    let mut xyzi: Vec<(f32, f32, f32, u16)> =
        (0..x.len()).map(|i| (x[i], y[i], z[i], i as u16)).collect();
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
pub fn solve_scan_aos_subscan([x, y, z]: [&[f32]; 3]) -> Vec<(u16, u16)> {
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
    let n = x.len();
    assert_eq!(n, x.len());
    assert_eq!(n, y.len());
    assert_eq!(n, z.len());
    assert!(n <= (u16::MAX - 10) as usize);

    let mut xyzi: Vec<(f32, f32, f32, u16)> =
        (0..x.len()).map(|i| (x[i], y[i], z[i], i as u16)).collect();
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
// 1. Move out std::thread::available_parallelism() (fs access is a bottleneck)
// 2. Avoid realloc on ans reducing
// 3. Input data as AoS rather than SoA
pub fn solve_scan_aos_subscan_threaded([x, y, z]: [&[f32]; 3]) -> Vec<(u16, u16)> {
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
    let n = x.len();
    assert_eq!(n, x.len());
    assert_eq!(n, y.len());
    assert_eq!(n, z.len());
    assert!(n <= (u16::MAX - 10) as usize);

    let mut xyzi: Vec<(f32, f32, f32, u16)> =
        (0..x.len()).map(|i| (x[i], y[i], z[i], i as u16)).collect();
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
    let parallel = std::thread::available_parallelism().unwrap().get();
    (0..parallel)
        .into_par_iter()
        .map(|chunk_idx| {
            let chunk_size = (xyzi.len() / parallel).max(1).min(xyzi.len());
            let start = chunk_idx * chunk_size;
            let end = if chunk_idx + 1 == parallel {
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
        .reduce(Vec::new, |mut a, b| {
            if a.is_empty() {
                b
            } else {
                a.extend_from_slice(&b);
                a
            }
        })
}

pub fn parse_input(mut source: impl Read) -> [Vec<f32>; 3] {
    let mut input = String::new();
    source.read_to_string(&mut input).unwrap();
    let mut x = Vec::new();
    let mut y = Vec::new();
    let mut z = Vec::new();
    for line in input.lines() {
        let mut nums = line.split(" ");
        x.push(nums.next().unwrap().parse().unwrap());
        y.push(nums.next().unwrap().parse().unwrap());
        z.push(nums.next().unwrap().parse().unwrap());
        assert_eq!(nums.next(), None);
    }
    [x, y, z]
}
pub fn closeness_1d(x: &[f32]) -> usize {
    let mut x = Vec::from(x);
    x.sort_unstable_by(|a, b| a.total_cmp(b));
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
pub fn compute_closeness([x, y, z]: [&[f32]; 3]) -> [usize; 3] {
    [closeness_1d(&x), closeness_1d(&y), closeness_1d(&z)]
}
