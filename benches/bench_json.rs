use criterion::{black_box, criterion_group, criterion_main, Criterion};

mod json_text_parse;

pub fn criterion_benchmark(c: &mut Criterion) {
    let input = std::fs::read_to_string("assets/data.json").unwrap();

    c.bench_function("parse json with text-parse", |b| {
        b.iter(|| json_text_parse::json().parse(black_box(&input)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
