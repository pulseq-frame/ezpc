use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::{collections::HashMap, str::FromStr};
use text_parse::*;

// NOTE: This code is taken from the crates.io example, not the pom benchmark!
// It is missing parsing for utf16 strings!

#[derive(Clone)]
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
    let special_char = tag("\\").val('\\')
        | tag("/").val('/')
        | tag("\"").val('"')
        | tag("b").val('\x08')
        | tag("f").val('\x0C')
        | tag("n").val('\n')
        | tag("r").val('\r')
        | tag("t").val('\t');
    let escape_sequence = tag("\\") + special_char;
    let string = tag("\"")
        + (none_of("\\\"").convert(|s| char::from_str(s)) | escape_sequence).repeat(0..)
        + tag("\"");
    string.map(|cs| cs.into_iter().collect())
}

fn array() -> Parser<impl Parse<Output = Vec<JsonValue>>> {
    let elems = list(value.wrap(), tag(",") + space());
    tag("[") + space() + elems + tag("]")
}

fn object() -> Parser<impl Parse<Output = HashMap<String, JsonValue>>> {
    let member = string() + space() + tag(":") + space() + value.wrap();
    let members = list(member, tag(",") + space());
    let obj = tag("{") + space() + members + tag("}");
    obj.map(|members| members.into_iter().collect::<HashMap<_, _>>())
}

fn value() -> Parser<impl Parse<Output = JsonValue>> {
    (tag("null").val(JsonValue::Null)
        | tag("true").val(JsonValue::Bool(true))
        | tag("false").val(JsonValue::Bool(false))
        | number().map(|num| JsonValue::Num(num))
        | string().map(|text| JsonValue::Str(text))
        | array().map(|arr| JsonValue::Array(arr))
        | object().map(|obj| JsonValue::Object(obj)))
        + space()
}

pub fn json() -> Parser<impl Parse<Output = JsonValue>> {
    space() + value()
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse number", |b| {
        b.iter(|| number().parse(black_box("-12.94e-5")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
