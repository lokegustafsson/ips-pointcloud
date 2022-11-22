use ips_pointcloud::{
    closeness_1d, parse_input, solve_naive, solve_scan, solve_scan_aos, solve_scan_aos_subscan,
};
use std::{io, time::Instant};

fn main() {
    let [x, y, z] = parse_input(io::stdin().lock());
    assert_eq!(x.len(), y.len());
    assert_eq!(x.len(), z.len());
    let xyz: [&[f32]; 3] = [&x, &y, &z];
    {
        let start = Instant::now();
        let xc = closeness_1d(&x);
        let yc = closeness_1d(&y);
        let zc = closeness_1d(&z);
        let us = Instant::now().duration_since(start).as_micros();
        dbg!(xc, yc, zc, us);
    }

    let mut ans = solve_naive(xyz);
    let mut answers = Vec::new();
    answers.push(run("solve_scan", solve_scan, xyz));
    answers.push(run("solve_scan_aos", solve_scan_aos, xyz));
    answers.push(run("solve_scan_aos_subscan", solve_scan_aos_subscan, xyz));
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
    fn run(
        msg: &str,
        solver: fn([&[f32]; 3]) -> Vec<(u16, u16)>,
        xyz: [&[f32]; 3],
    ) -> Vec<(u16, u16)> {
        const N: usize = 400;
        let start = Instant::now();
        let mut ret = solver(xyz);
        for _ in 0..(N - 1) {
            ret = solver(xyz);
        }
        println!(
            "{msg}:\t{:>6}us",
            Instant::now().duration_since(start).as_micros() / (N as u128)
        );
        ret
    }
}
