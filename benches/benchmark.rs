use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput,
};
use qlog::{info, Logger};

pub fn log_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("logging events");
    for size in [16, 256, 1024, 4096, 8192].into_iter() {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_function(BenchmarkId::from_parameter(size), |b| {
            b.iter_batched(
                || Logger::new(None),
                |logger| {
                    for _ in 0..size {
                        info!(logger, "{}", black_box(size));
                    }

                    logger
                },
                BatchSize::PerIteration,
            );
        });
    }
    group.finish();
}

criterion_group!(bench, log_bench);
criterion_main!(bench);
