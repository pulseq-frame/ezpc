use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::{collections::HashMap, str::FromStr};
use text_parse::*;

pub enum JsonValue {
    Null,
    Bool(bool),
    Str(String),
    Num(f64),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

fn space() -> Matcher<impl Match> {
    one_of(" \t\r\n").repeat(0..)
}

fn number() -> Parser<impl Parse<Output = f64>> {
    let integer = (one_of("123456789") + one_of("0123456789").repeat(0..)) | tag("0");
    let frac = tag(".") + one_of("0123456789").repeat(1..);
    let exp = one_of("eE") + one_of("+-").opt() + one_of("0123456789").repeat(1..);
    let number = tag("-").opt() + integer + frac.opt() + exp.opt();
    number.convert(|s| f64::from_str(&s))
}

fn string() -> Parser<impl Parse<Output = String>> {
    let special_char = tag("\\")
        | tag("/")
        | tag("\"")
        | tag("b").map(|_| b'\x08')
        | tag("f").map(|_| b'\x0C')
        | tag("n").map(|_| b'\n')
        | tag("r").map(|_| b'\r')
        | tag("t").map(|_| b'\t');
    let escape_sequence = tag("\\") + special_char;
    let string = tag("\"") + (none_of("\\\"") | escape_sequence).repeat(0..) + tag("\"");
    string.convert(String::from_utf8)
}

fn array() -> Parser<impl Parse<Output = Vec<JsonValue>>> {
    let elems = list(call(value), tag(",") + space());
    tag("[") + space() + elems + tag("]")
}

fn object() -> Parser<impl Parse<Output = HashMap<String, JsonValue>>> {
    let member = string() + space() + tag(":") + space() + call(value);
    let members = list(member, tag(",") + space());
    let obj = tag("{") + space() + members + tag("}");
    obj.map(|members| members.into_iter().collect::<HashMap<_, _>>())
}

fn value() -> Parser<impl Parse<Output = JsonValue>> {
    (tag("null").map(|_| JsonValue::Null)
        | tag("true").map(|_| JsonValue::Bool(true))
        | tag("false").map(|_| JsonValue::Bool(false))
        | number().map(|num| JsonValue::Num(num))
        | string().map(|text| JsonValue::Str(text))
        | array().map(|arr| JsonValue::Array(arr))
        | object().map(|obj| JsonValue::Object(obj)))
        + space()
}

pub fn json() -> Parser<impl Parse<Output =  JsonValue>> {
    space() + value()
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse number", |b| {
        b.iter(|| number().parse(black_box("-12.94e-5")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
