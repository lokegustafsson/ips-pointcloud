use ips_pointcloud::{
    compute_closeness, parse_input, slice_assume_init, solve_naive, solve_threaded, ScanSolver,
    SubscanSolver,
};
use std::{cell::UnsafeCell, time::Instant};

const DATA: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/positions.xyz"));
const COMPUTE_CLOSENESS: bool = true;
const SCAN_ITERS: usize = 3000;
const SUBSCAN_ITERS: usize = 3000;

fn main() {
    let xyzi = &parse_input(DATA);
    let parallel = std::thread::available_parallelism().unwrap();
    dbg!(parallel);
    if COMPUTE_CLOSENESS {
        let start = Instant::now();
        let [xc, yc, zc] = compute_closeness(xyzi);
        let us = Instant::now().duration_since(start).as_micros();
        // Scan and Subscan works best if closeness is increasing with x,y,z
        // We can reflect the points without changing the answers.
        println!("close pairs (x,y,z): {xc} {yc} {zc} ({us}us)");
    }

    let mut ans = solve_naive(xyzi);
    let mut answers = Vec::new();

    // solve_threaded_scan: Threaded 1D pass, trying all pairs with close x-vals
    let mut scan_xyzi = Vec::new();
    let mut x_soa = Vec::new();
    let mut y_soa = Vec::new();
    let mut z_soa = Vec::new();
    let mut i_soa = Vec::new();
    let mut scan_ret = UnsafeCell::new(Vec::new());
    answers.push({
        run::<SCAN_ITERS>("solve_threaded_scan", || {
            scan_xyzi.truncate(0);
            scan_xyzi.extend_from_slice(xyzi);
            solve_threaded::<ScanSolver>(
                &mut scan_xyzi,
                (&mut x_soa, &mut y_soa, &mut z_soa, &mut i_soa),
                parallel,
                &mut scan_ret,
            );
        });
        unsafe { slice_assume_init(scan_ret.get_mut()) }
    });

    // solve_threaded_subscan: Threaded 2D pass, trying all pairs with close x-vals & y-vals.
    let mut subscan_xyzi = Vec::new();
    let mut x_soa = Vec::new();
    let mut y_soa = Vec::new();
    let mut z_soa = Vec::new();
    let mut i_soa = Vec::new();
    let mut subscan_ret = UnsafeCell::new(Vec::new());
    answers.push({
        run::<SUBSCAN_ITERS>("solve_threaded_subscan", || {
            subscan_xyzi.truncate(0);
            subscan_xyzi.extend_from_slice(xyzi);
            solve_threaded::<SubscanSolver>(
                &mut subscan_xyzi,
                (&mut x_soa, &mut y_soa, &mut z_soa, &mut i_soa),
                parallel,
                &mut subscan_ret,
            );
        });
        unsafe { slice_assume_init(subscan_ret.get_mut()) }
    });
    {
        println!("Neighbor count: {}", ans.len());
        ans.sort_unstable();
        for (i, a) in answers.iter_mut().enumerate() {
            a.sort_unstable();
            assert_eq!(ans.len(), a.len(), "{i}");
            for j in 0..ans.len() {
                assert_eq!(ans[j], a[j], "{i}, {j}");
            }
        }
    }
}
fn run<const N: usize>(msg: &str, mut solver: impl FnMut()) {
    let start = Instant::now();
    for _ in 0..N {
        solver();
    }
    println!(
        "{msg}:\t{:>6}us",
        Instant::now().duration_since(start).as_micros() / (N as u128)
    );
}
