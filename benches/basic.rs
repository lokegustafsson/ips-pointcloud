use criterion::Criterion;
use ips_pointcloud::{
    compute_closeness, parse_input, solve_scan, solve_scan_aos, solve_scan_aos_subscan,
    solve_scan_aos_subscan_threaded,
};

const DATA: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/data.in"));

fn criterion_benchmark(c: &mut Criterion) {
    let xyzi = &parse_input(DATA);

    c.bench_function("compute_closeness", |b| {
        b.iter(|| compute_closeness(criterion::black_box(xyzi)))
    });
    c.bench_function("solve_scan", |b| {
        b.iter(|| solve_scan(criterion::black_box(xyzi)))
    });
    c.bench_function("solve_scan_aos", |b| {
        b.iter(|| solve_scan_aos(criterion::black_box(xyzi)))
    });
    c.bench_function("solve_scan_aos_subscan", |b| {
        b.iter(|| solve_scan_aos_subscan(criterion::black_box(xyzi)))
    });
    c.bench_function("solve_scan_aos_subscan_threaded", |b| {
        b.iter(|| solve_scan_aos_subscan_threaded(criterion::black_box(xyzi)))
    });
}

criterion::criterion_group!(benches, criterion_benchmark);
criterion::criterion_main!(benches);
