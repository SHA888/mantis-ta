use criterion::{criterion_group, criterion_main, Criterion};

fn bench_noop(c: &mut Criterion) {
    c.bench_function("strategy_noop", |b| b.iter(|| 0u8));
}

criterion_group!(strategy_eval, bench_noop);
criterion_main!(strategy_eval);
