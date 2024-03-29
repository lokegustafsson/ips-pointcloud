use criterion::Criterion;
use ips_pointcloud::{compute_closeness, parse_input, solve_threaded, ScanSolver, SubscanSolver};
use std::cell::UnsafeCell;

const DATA: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/positions.xyz"));

fn criterion_benchmark(c: &mut Criterion) {
    let xyzi = &parse_input(DATA);
    let parallel = std::thread::available_parallelism().unwrap();
    let mut scan_ret = UnsafeCell::new(Vec::new());
    let mut subscan_ret = UnsafeCell::new(Vec::new());

    let mut x_soa = Vec::new();
    let mut y_soa = Vec::new();
    let mut z_soa = Vec::new();
    let mut i_soa = Vec::new();

    let mut x_soa2 = Vec::new();
    let mut y_soa2 = Vec::new();
    let mut z_soa2 = Vec::new();
    let mut i_soa2 = Vec::new();

    c.bench_function("compute_closeness", |b| {
        b.iter(|| compute_closeness(criterion::black_box(xyzi)))
    });
    c.bench_function("solve_threaded_scan", |b| {
        b.iter(|| {
            let mut xyzi = criterion::black_box(xyzi).to_owned();
            solve_threaded::<ScanSolver>(
                &mut xyzi,
                (&mut x_soa, &mut y_soa, &mut z_soa, &mut i_soa),
                parallel,
                &mut scan_ret,
            )
        })
    });
    c.bench_function("solve_threaded_subscan", |b| {
        b.iter(|| {
            let mut xyzi = criterion::black_box(xyzi).to_owned();
            solve_threaded::<SubscanSolver>(
                &mut xyzi,
                (&mut x_soa2, &mut y_soa2, &mut z_soa2, &mut i_soa2),
                parallel,
                &mut subscan_ret,
            )
        })
    });
}

criterion::criterion_group!(benches, criterion_benchmark);
criterion::criterion_main!(benches);
