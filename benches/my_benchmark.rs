use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::str::FromStr;
use text_parse::*;

fn integer() -> Matcher<impl Match> {
    tag("0") | (one_of("123456789") + one_of("0123456789").repeat(0..))
}

fn number() -> Parser<impl Parse<Output = f64>> {
    let frac = tag(".") + one_of("0123456789").repeat(1..);
    let exp = one_of("eE") + one_of("+-").opt() + one_of("0123456789").repeat(1..);
    let number = tag("-").opt() + integer() + frac.opt() + exp.opt();

    number.map_match(f64::from_str)
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse number", |b| {
        b.iter(|| number().parse(black_box("-12.94e-5")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
