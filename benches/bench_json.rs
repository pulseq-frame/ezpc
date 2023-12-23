use criterion::{black_box, criterion_group, criterion_main, Criterion};

mod json_pom;
mod json_ezpc;

fn json(c: &mut Criterion) {
    let input = std::fs::read_to_string("assets/data.json").unwrap();
    let mut group = c.benchmark_group("json");

    group.bench_function("pom", |b| {
        b.iter(|| {
            json_pom::json().parse(black_box(input.as_bytes())).ok();
        })
    });

    group.bench_function("text-parse", |b| {
        b.iter(|| {
            json_ezpc::json().parse_all(black_box(&input)).ok();
        })
    });

    group.finish();
}

criterion_group!(benches, json);
criterion_main!(benches);
