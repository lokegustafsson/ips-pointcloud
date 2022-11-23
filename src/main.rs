use ips_pointcloud::{
    compute_closeness, parse_input, slice_assume_init, solve_naive, solve_scan, solve_subscan,
    solve_subscan_threaded,
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
    let mut solve_subscan_threaded_ret = Vec::new();
    //answers.push(run("solve_scan", || solve_scan(&xyzi)));
    //answers.push(run("solve_subscan", || {
    //    solve_subscan(&xyzi)
    //}));
    answers.push({
        run("solve_subscan_threaded", || {
            let mut xyzi = xyzi.clone();
            solve_subscan_threaded(&mut xyzi, parallel, &mut solve_subscan_threaded_ret);
        });
        unsafe { slice_assume_init(solve_subscan_threaded_ret.as_mut()) }
    });
    println!("Neighbor count: {}", ans.len());
    {
        ans.sort_unstable();
        for (i, a) in answers.iter_mut().enumerate() {
            a.sort_unstable();
            assert_eq!(ans.len(), a.len(), "{i}");
            for j in 0..ans.len() {
                assert_eq!(ans[j], a[j], "{i}, {j}");
            }
        }
    }
    fn run<'a>(msg: &str, mut solver: impl FnMut()) {
        const N: usize = 500;
        let start = Instant::now();
        for _ in 0..N {
            solver();
        }
        println!(
            "{msg}:\t{:>6}us",
            Instant::now().duration_since(start).as_micros() / (N as u128)
        );
    }
}
