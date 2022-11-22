use ips_pointcloud::{
    compute_closeness, parse_input, solve_naive, solve_scan, solve_scan_aos,
    solve_scan_aos_subscan, solve_scan_aos_subscan_threaded,
};
use std::{io, time::Instant};

fn main() {
    let xyzi = parse_input(io::stdin().lock());
    let parallel = std::thread::available_parallelism().unwrap();
    {
        let start = Instant::now();
        let [xc, yc, zc] = compute_closeness(&xyzi);
        let us = Instant::now().duration_since(start).as_micros();
        dbg!(xc, yc, zc, us);
    }

    let mut ans = solve_naive(&xyzi);
    let mut answers = Vec::new();
    answers.push(run("solve_scan", || solve_scan(&xyzi)));
    answers.push(run("solve_scan_aos", || solve_scan_aos(&xyzi)));
    answers.push(run("solve_scan_aos_subscan", || {
        solve_scan_aos_subscan(&xyzi)
    }));
    answers.push(run("solve_scan_aos_subscan_threaded", || {
        solve_scan_aos_subscan_threaded(&xyzi, parallel)
    }));
    println!("Neighbor count: {}", ans.len());
    {
        ans.sort_unstable();
        for (i, a) in answers.iter_mut().enumerate() {
            a.sort_unstable();
            assert_eq!(ans.len(), a.len(), "{i}");
            for j in 0..ans.len() {
                assert_eq!(ans[i], a[i], "{i}, {j}");
            }
        }
    }
    fn run(msg: &str, solver: impl Fn() -> Vec<(u16, u16)>) -> Vec<(u16, u16)> {
        const N: usize = 1000;
        let start = Instant::now();
        let mut ret = solver();
        for _ in 0..(N - 1) {
            ret = solver();
        }
        println!(
            "{msg}:\t{:>6}us",
            Instant::now().duration_since(start).as_micros() / (N as u128)
        );
        ret
    }
}
