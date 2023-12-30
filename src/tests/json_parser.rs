use crate::*;
use std::str::FromStr;

// Modelled after the official JSON grammar, see: json_grammar.txt

#[derive(Debug, Clone)]
pub enum JsonValue {
    Object(Vec<(String, JsonValue)>),
    Array(Vec<JsonValue>),
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

pub fn json() -> Parser<impl Parse<Output = JsonValue>> {
    element()
}

fn value() -> Parser<impl Parse<Output = JsonValue>> {
    object().map(|o| JsonValue::Object(o))
        | array().map(|a| JsonValue::Array(a))
        | string().map(|s| JsonValue::String(s))
        | number().map(|x| JsonValue::Number(x))
        | tag("true").val(JsonValue::Bool(true))
        | tag("false").val(JsonValue::Bool(false))
        | tag("null").val(JsonValue::Null)
}

fn object() -> Parser<impl Parse<Output = Vec<(String, JsonValue)>>> {
    (tag("{") + ws() + tag("}")).val(Vec::new()) | (tag("{") + members() + tag("}"))
}

fn members() -> Parser<impl Parse<Output = Vec<(String, JsonValue)>>> {
    list(member(), tag(","))
}

fn member() -> Parser<impl Parse<Output = (String, JsonValue)>> {
    ws() + string() + ws() + tag(":") + element()
}

fn array() -> Parser<impl Parse<Output = Vec<JsonValue>>> {
    tag("[") + !((elements() + tag("]")) | (ws().val(Vec::new()) + tag("]"))).name("values or ']'")
}

fn elements() -> Parser<impl Parse<Output = Vec<JsonValue>>> {
    list(element(), tag(","))
}

fn element() -> Parser<impl Parse<Output = JsonValue>> {
    ws() + !value.wrap(100).name("value") + ws()
}

fn string() -> Parser<impl Parse<Output = String>> {
    (tag("\"") + characters() + tag("\"")).map(|chars| chars.into_iter().collect())
}

fn characters() -> Parser<impl Parse<Output = Vec<char>>> {
    // In contrast to the spec, we parse utf16 characters separate to the
    // escaped characters, because a single utf16 code point might not be a
    // legal `char` (utf-32 code point). So the separate utf16 parser parses
    // all consecutive \uXXXX escape sequences and splits it then into `char`s
    (utf16_chars() | character().repeat(1..))
        .repeat(0..)
        .map(|nested| nested.into_iter().flatten().collect())
}

fn character() -> Parser<impl Parse<Output = char>> {
    // The spec allows all characters between 0x20 and 0x10FFFF, but "surrogate
    // code points" used by UTF-16 (0xD800 to 0xDFFF) are not valid code points
    // for a single char. Rust strings are always valid unicode, so we parse
    // everything except control characters (0x0 to 0x1F) as valid char.
    is_a("utf-16 codepoint", |c| {
        !matches!(c, '\0'..='\u{1F}' | '"' | '\\')
    })
    .map(|s| char::from_str(s).unwrap())
        | (tag("\\") + escape())
}

fn escape() -> Parser<impl Parse<Output = char>> {
    tag("\"").val('"')
        | tag("\\").val('\\')
        | tag("/").val('/')
        | tag("b").val('\x08')
        | tag("f").val('\x0C')
        | tag("n").val('\n')
        | tag("r").val('\r')
        | tag("t").val('\t')
    // 'u' hex hex hex hex already handled by utf16 chars
}

fn utf16_chars() -> Parser<impl Parse<Output = Vec<char>>> {
    (tag("\\u") + hex().repeat(4).map(|s| u16::from_str_radix(s, 16).unwrap()))
        .repeat(1..)
        .try_map(|utf16| char::decode_utf16(utf16.into_iter()).collect())
}

fn hex() -> Matcher<impl Match> {
    digit() | is_a("[A-F]", |c| matches!(c, 'A'..='F')) | is_a("[a-f]", |c| matches!(c, 'a'..='f'))
}

fn number() -> Parser<impl Parse<Output = f64>> {
    (integer() + fraction() + exponent()).map(|s: &str| f64::from_str(s).unwrap())
}

fn integer() -> Matcher<impl Match> {
    // Small change from grammar: try to parse the longer matcher first,
    // otherwise it fails when the integer continues.
    (onenine() + digits()) | digit() | (tag("-") + onenine() + digits()) | (tag("-") + digit())
}

fn digits() -> Matcher<impl Match> {
    digit().repeat(1..)
}

fn digit() -> Matcher<impl Match> {
    tag("0") | onenine()
}

fn onenine() -> Matcher<impl Match> {
    is_a("[1-9]", |c| matches!(c, '1'..='9'))
}

fn fraction() -> Matcher<impl Match> {
    (tag(".") + digits()).opt()
}

fn exponent() -> Matcher<impl Match> {
    (one_of("Ee") + sign() + digits()).opt()
}

fn sign() -> Matcher<impl Match> {
    one_of("+-").opt()
}

fn ws() -> Matcher<impl Match> {
    one_of("\u{20}\u{A}\u{D}\u{9}").repeat(0..)
}
