use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use voting_map::{election, Point};

fn criterion_benchmark(c: &mut Criterion) {
    let candidates = vec![
        Point::new(0.12, 0.28),
        Point::new(0.85, 0.7),
        Point::new(0.39, 0.28),
        Point::new(0.97, 0.14),
    ];

    let mut group = c.benchmark_group("election");
    for size in [128].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| election(size, &candidates, "hare"))
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
