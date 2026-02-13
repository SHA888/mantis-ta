use criterion::{criterion_group, criterion_main, Criterion};

fn bench_noop(c: &mut Criterion) {
    c.bench_function("backtest_noop", |b| b.iter(|| 0u8));
}

criterion_group!(backtest, bench_noop);
criterion_main!(backtest);
