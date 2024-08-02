use blog::match_route;
use criterion::{criterion_group, criterion_main, Criterion};

fn test_one(route: &str, format: &str) {
    match_route(route, format);
}

fn parsing_benchmark(c: &mut Criterion) {
    c.bench_function("basic post route test", |b| {
        b.iter(|| test_one("/post/david", "/post/:slug"))
    });

    c.bench_function("basic home route test", |b| {
        b.iter(|| test_one("/home", "/home"))
    });
}

criterion_group!(benches, parsing_benchmark);
criterion_main!(benches);
