use criterion::Criterion;
use ips_pointcloud::{
    compute_closeness, parse_input, solve_scan, solve_subscan, solve_subscan_threaded,
};

const DATA: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/data.in"));

fn criterion_benchmark(c: &mut Criterion) {
    let xyzi = &parse_input(DATA);
    let parallel = std::thread::available_parallelism().unwrap();
    let mut solve_subscan_threaded_ret = Vec::new();

    c.bench_function("compute_closeness", |b| {
        b.iter(|| compute_closeness(criterion::black_box(xyzi)))
    });
    c.bench_function("solve_scan", |b| {
        b.iter(|| solve_scan(criterion::black_box(xyzi)))
    });
    c.bench_function("solve_subscan", |b| {
        b.iter(|| solve_subscan(criterion::black_box(xyzi)))
    });
    c.bench_function("solve_subscan_threaded", |b| {
        b.iter(|| {
            let mut xyzi = criterion::black_box(xyzi).to_owned();
            solve_subscan_threaded(&mut xyzi, parallel, &mut solve_subscan_threaded_ret)
        })
    });
}

criterion::criterion_group!(benches, criterion_benchmark);
criterion::criterion_main!(benches);
