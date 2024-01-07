use ezpc::*;
use std::{collections::HashMap, str::FromStr};

#[derive(Clone, Debug)]
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
    number.try_map(f64::from_str)
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
    let char_string = (none_of("\\\"").try_map(char::from_str) | escape_sequence)
        .repeat(1..)
        .map(|cs| cs.into_iter().collect::<String>());
    let utf16_char = tag("\\u")
        + is_a(|c| c.is_ascii_hexdigit())
            .repeat(4)
            .try_map(|digits| u16::from_str_radix(digits, 16));
    let utf16_string = utf16_char.repeat(1..).map(|chars| {
        std::char::decode_utf16(chars)
            .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect::<String>()
    });
    let string = tag("\"") + (char_string | utf16_string).repeat(0..) + tag("\"");
    string.map(|strings| strings.concat())
}

fn array() -> Parser<impl Parse<Output = Vec<JsonValue>>> {
    let elems = list(value.wrap(100), tag(",") + space());
    tag("[") + space() + elems + tag("]")
}

fn object() -> Parser<impl Parse<Output = HashMap<String, JsonValue>>> {
    let member = string() + space() + tag(":") + space() + value.wrap(100);
    let members = list(member, tag(",") + space());
    let obj = tag("{") + space() + members + tag("}");
    obj.map(|members| members.into_iter().collect::<HashMap<_, _>>())
}

fn value() -> Parser<impl Parse<Output = JsonValue>> {
    (tag("null").val(JsonValue::Null)
        | tag("true").val(JsonValue::Bool(true))
        | tag("false").val(JsonValue::Bool(false))
        | number().map(JsonValue::Num)
        | string().map(JsonValue::Str)
        | array().map(JsonValue::Array)
        | object().map(JsonValue::Object))
        + space()
}

pub fn json() -> Parser<impl Parse<Output = JsonValue>> {
    space() + value()
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_json() {
        let input = std::fs::read_to_string("assets/data.json").unwrap();
        println!("{:#?}", super::json().parse_all(&input));
        assert!(super::json().parse_all(&input).is_ok());
    }
}
