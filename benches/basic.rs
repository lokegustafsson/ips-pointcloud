use criterion::Criterion;
use ips_pointcloud::{compute_closeness, parse_input, solve_threaded, ScanSolver, SubscanSolver};

const DATA: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/positions.xyz"));

fn criterion_benchmark(c: &mut Criterion) {
    let xyzi = &parse_input(DATA);
    let parallel = std::thread::available_parallelism().unwrap();
    let mut scan_ret = Vec::new();
    let mut subscan_ret = Vec::new();

    c.bench_function("compute_closeness", |b| {
        b.iter(|| compute_closeness(criterion::black_box(xyzi)))
    });
    c.bench_function("solve_threaded_scan", |b| {
        b.iter(|| {
            let mut xyzi = criterion::black_box(xyzi).to_owned();
            solve_threaded::<ScanSolver>(&mut xyzi, parallel, &mut scan_ret)
        })
    });
    c.bench_function("solve_threaded_subscan", |b| {
        b.iter(|| {
            let mut xyzi = criterion::black_box(xyzi).to_owned();
            solve_threaded::<SubscanSolver>(&mut xyzi, parallel, &mut subscan_ret)
        })
    });
}

criterion::criterion_group!(benches, criterion_benchmark);
criterion::criterion_main!(benches);
