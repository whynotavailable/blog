use blog::match_route;
use criterion::{criterion_group, criterion_main, Criterion};

fn test_one() {
    match_route("/post/david", "/post/:slug");
}

fn parsing_benchmark(c: &mut Criterion) {
    c.bench_function("basic post route test", |b| b.iter(test_one));
}

criterion_group!(benches, parsing_benchmark);
criterion_main!(benches);
