use crate::*;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum JsonValue {
    Object(Vec<(String, JsonValue)>),
    Array(Vec<JsonValue>),
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

pub fn json() -> Parser<impl Parse<Output = JsonValue>> {
    value()
}

fn value_inner() -> Parser<impl Parse<Output = JsonValue>> {
    object().map(|o| JsonValue::Object(o))
        | array().map(|a| JsonValue::Array(a))
        | string().map(|s| JsonValue::String(s))
        | number().map(|x| JsonValue::Number(x))
        | tag("true").val(JsonValue::Bool(true))
        | tag("false").val(JsonValue::Bool(false))
        | tag("null").val(JsonValue::Null)
}

fn value() -> Parser<impl Parse<Output = JsonValue>> {
    // Whenever we try to parse a value it is a fatal error if we fail:
    // If a value is expected, we know for sure there should be one and nothing else
    ws() + value_inner.wrap(100).fatal(error_msg::UNKNOWN_VALUE) + ws()
}

fn object() -> Parser<impl Parse<Output = Vec<(String, JsonValue)>>> {
    let member = ws() + string() + ws() + tag(":") + value();
    let members = list(member, tag(","), error_msg::MISSING_OBJECT_MEMBER);
    tag("{")
        + ((ws() + tag("}")).val(Vec::new()) | (members + tag("}")))
            .fatal(error_msg::UNCLOSED_OBJECT)
}

fn array() -> Parser<impl Parse<Output = Vec<JsonValue>>> {
    let elements = list(value(), tag(","), error_msg::MISSING_ARRAY_ELEMENT);
    tag("[")
        + ((ws() + tag("]")).val(Vec::new()) | (elements + tag("]")))
            .fatal(error_msg::UNCLOSED_ARRAY)
}

fn integer() -> Matcher<impl Match> {
    (tag("0") + one_of("0123456789").reject(error_msg::LEADING_ZERO))
        | one_of("123456789") + one_of("0123456789").repeat(0..)
}

fn number() -> Parser<impl Parse<Output = f64>> {
    let frac = tag(".") + one_of("0123456789").repeat(1..);
    let exp = one_of("eE") + one_of("+-").opt() + one_of("0123456789").repeat(1..);
    (tag("-").opt() + integer() + frac.opt() + exp.opt())
        .convert(f64::from_str, error_msg::PARSE_ERROR)
}

fn string() -> Parser<impl Parse<Output = String>> {
    (tag("\"")
        + (char_str() | utf16_str() | esc_str()).repeat(0..)
        + tag("\"").fatal(error_msg::UNCLOSED_STRING))
    .map(|strs| strs.concat())
}

fn char_str() -> Parser<impl Parse<Output = String>> {
    (is_a(|c| matches!(c, '\0'..='\u{1F}')).reject(error_msg::UNESCAPED_CTRL_CHAR)
        + none_of("\\\""))
    .repeat(1..)
    .map(|s| s.to_owned())
}

fn utf16_str() -> Parser<impl Parse<Output = String>> {
    let hex = is_a(|c| matches!(c, '0'..='9' | 'a'..='f' | 'A'..='F'))
        .repeat(4)
        .map(|s| u16::from_str_radix(s, 16).unwrap());
    (tag("\\u") + hex).repeat(1..).convert(
        |utf16| char::decode_utf16(utf16.into_iter()).collect(),
        error_msg::ILLEGAL_UTF16,
    )
}

fn esc_str() -> Parser<impl Parse<Output = String>> {
    let esc = tag("\"").val("\"")
        | tag("\\").val("\\")
        | tag("/").val("/")
        | tag("b").val("\x08")
        | tag("f").val("\x0C")
        | tag("n").val("\n")
        | tag("r").val("\r")
        | tag("t").val("\t");

    (tag("\\") + esc.fatal(error_msg::ESCAPE_SEQUENCE))
        .repeat(1..)
        .map(|strs| strs.concat())
}

fn ws() -> Matcher<impl Match> {
    one_of("\n\r\t ").repeat(0..)
}

mod error_msg {
    pub(super) const UNCLOSED_STRING: &str = "Missing trailing '\"' to close string literal:";
    pub(super) const UNCLOSED_ARRAY: &str = "Missing trailing ']' to close array:";
    pub(super) const UNCLOSED_OBJECT: &str = "Missing trailing '}' to close object:";
    pub(super) const ESCAPE_SEQUENCE: &str =
        "Illegal escape sequence: Only \"\\/bfrnrt are allowed:";
    pub(super) const ILLEGAL_UTF16: &str = "Illegal utf-16 string:";
    pub(super) const LEADING_ZERO: &str = "Integer cannot start with a leading zero:";
    pub(super) const UNKNOWN_VALUE: &str = "Failed to parse expected value:";
    pub(super) const UNESCAPED_CTRL_CHAR: &str = "Illegal unescaped control character:";
    pub(super) const PARSE_ERROR: &str = "Internal error: failed to parse matched string:";
    pub(super) const MISSING_ARRAY_ELEMENT: &str = "Expected an array element:";
    pub(super) const MISSING_OBJECT_MEMBER: &str = "Expected an object member:";
}
