use std::str::FromStr;
use std::{error::Error, fmt::Display};

use ezpc::*;

// NOTES:
// Sometimes, we want a temporary parser inside of a function.
// If we want to reuse it, we can't clone it:
// This scenario seems rare, use a clouser (e.g.: let parser = || tag("hi");)
// which also means that the usage is consistent with those defined in functions.
// Maybe it would be still nice to provide a clone() function on all parsers / matchers.

fn main() {
    let src = r##"
    
int1 = +99
int2 = 42
int3 = 0
int4 = -17
int5 = 1_000
int6 = 5_349_221
int7 = 53_49_221  # Indian number system grouping
int8 = 1_2_3_4_5  # VALID but discouraged

# hexadecimal with prefix `0x`
hex1 = 0xDEADBEEF
hex2 = 0xdeadbeef
hex3 = 0xdead_beef

# octal with prefix `0o`
oct1 = 0o01234567
oct2 = 0o755 # useful for Unix file permissions

# binary with prefix `0b`
bin1 = 0b11010110
    
# fractional
flt1 = +1.0
flt2 = 3.1415
flt3 = -0.01

# exponent
flt4 = 5e+22
flt5 = 1e06
flt6 = -2E-2

# both
flt7 = 6.626e-34

flt8 = 224_617.445_991_228

# infinity
sf1 = inf  # positive infinity
sf2 = +inf # positive infinity
sf3 = -inf # negative infinity

# not a number
sf4 = nan  # actual sNaN/qNaN encoding is implementation-specific
sf5 = +nan # same as `nan`
sf6 = -nan # valid, actual encoding is implementation-specific

bool1 = true
bool2 = false

"##;

    match file().parse_all(src) {
        Ok(stmts) => {
            for stmt in stmts {
                println!("{stmt:?}")
            }
        }
        Err(err) => println!("{err}"),
    }
}

#[derive(Clone, Debug)]
pub struct Key {
    // This vec contains the elements of a "dotted key" (single elements if no dot)
    path: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

#[derive(Clone, Debug)]
pub enum Statement {
    // Array
    // Table
    KeyValue { key: Key, value: Value },
}

pub fn file() -> Parser<impl Parse<Output = Vec<Statement>>> {
    ln().opt() + statement().repeat(0..)
}

fn statement() -> Parser<impl Parse<Output = Statement>> {
    // Add Arrays and Tables
    (
        key_value().map(|(key, value)| Statement::KeyValue { key, value })
        // | table()
        // | array()
    ) + ln().fatal(error_msg::NO_NEWLINE)
}

fn key_value() -> Parser<impl Parse<Output = (Key, Value)>> {
    key() + ws() + tag("=") + ws() + value()
}

fn key() -> Parser<impl Parse<Output = Key>> {
    ((bare_key() | quoted_key()) + (tag(".") + (bare_key() | quoted_key())).repeat(0..)).map(
        |(head, mut tail)| {
            tail.insert(0, head);
            Key { path: tail }
        },
    )
}

fn bare_key() -> Parser<impl Parse<Output = String>> {
    // TODO: Add a .collect() function to map to String (often used!)
    is_a(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        .repeat(1..)
        .map(|s| s.to_owned())
}

fn quoted_key() -> Parser<impl Parse<Output = String>> {
    basic_string() | literal_string()
}

fn value() -> Parser<impl Parse<Output = Value>> {
    (tag("true").val(Value::Boolean(true))
        | tag("false").val(Value::Boolean(false))
        | float().map(Value::Float)
        | integer().map(Value::Integer)
        | multiline_basic_string().map(Value::String)
        | basic_string().map(Value::String)
        | literal_string().map(Value::String))
    .fatal(error_msg::UNKNOWN_VALUE)
}

// --------------
// Number parsing
// --------------

fn integer() -> Parser<impl Parse<Output = i64>> {
    let sign = || {
        one_of("+-").opt().map(|x| match x {
            "" | "+" => 1,
            "-" => -1,
            _ => unreachable!(),
        })
    };
    // TODO: some easy way of saying that one parser should match, but not another would be helpful:
    // could write that is_a should match, but not "0"
    // TODO: provide digit(radix) parser with ezpc as it's often used
    let digit = |radix| is_a(move |c| c.is_digit(radix));
    // TODO: provide not("...") parser as opposite of tag(), like none_of to one_of
    // TODO: It would be _really_ nice if we could write something like (digit(10) & not("0")): only match if both matchers apply
    // TODO: documentation of all ezpc methods
    // Modified version of i64::from_str_radix that filters out underscores
    let from_str_radix = |src: &str, radix: u32| {
        let filtered: String = src.chars().filter(|c| *c != '_').collect();
        i64::from_str_radix(&filtered, radix)
    };
    // Decimal numbers are not allowed leading zeros
    let number_dec = || {
        let raw = (tag("0") + (tag("_") | digit(10)).reject(error_msg::LEADING_ZERO))
            | (digit(10) + (tag("_").opt() + digit(10)).repeat(0..));
        raw.convert(move |s| from_str_radix(s, 10), error_msg::NUMBER_TOO_BIG)
    };
    let number = |radix| {
        let raw = digit(radix) + (tag("_").opt() + digit(radix)).repeat(0..);
        raw.convert(move |s| from_str_radix(s, radix), error_msg::NUMBER_TOO_BIG)
    };

    let bin = (sign() + tag("0b") + number(2)).map(|(sign, num)| sign * num);
    let oct = (sign() + tag("0o") + number(8)).map(|(sign, num)| sign * num);
    let hex = (sign() + tag("0x") + number(16)).map(|(sign, num)| sign * num);
    let dec = (sign() + number_dec()).map(|(sign, num)| sign * num);

    bin | oct | hex | dec
}

fn float() -> Parser<impl Parse<Output = f64>> {
    let digit = || is_a(|c| c.is_ascii_digit());
    let int_no_leading_zero = || {
        (tag("0") + (tag("_") | digit()).reject(error_msg::LEADING_ZERO))
            | (digit() + (tag("_").opt() + digit()).repeat(0..))
    };
    let int = || digit() + (tag("_").opt() + digit()).repeat(0..);

    let exp = || one_of("eE") + one_of("+-").opt() + int();
    let fract = (tag(".") + int() + exp().opt()) | exp();
    let matcher = one_of("+-").opt() + int_no_leading_zero() + fract;

    let special = (tag("inf") | tag("+inf")).val(f64::INFINITY)
        | tag("-inf").val(f64::NEG_INFINITY)
        | (one_of("+-").opt() + tag("nan")).val(f64::NAN);

    special
        | matcher.convert(
            |s| {
                let filtered: String = s.chars().filter(|c| *c != '_').collect();
                f64::from_str(&filtered)
            },
            error_msg::PARSE_ERROR,
        )
}

// --------------
// String parsing
// --------------

fn literal_string() -> Parser<impl Parse<Output = String>> {
    tag("'")
        + none_of("\r\n'").repeat(1..).map(|s| s.to_owned())
        + tag("'").fatal(error_msg::UNCLOSED_STRING)
}

fn basic_string() -> Parser<impl Parse<Output = String>> {
    (tag("\"")
        + (char_str() | utf32_str() | utf16_str() | esc_str()).repeat(0..)
        + tag("\"").fatal(error_msg::UNCLOSED_STRING))
    .map(|strs| strs.concat())
}

fn multiline_basic_string() -> Parser<impl Parse<Output = String>> {
    (tag(r#"""""#)
        + (tag("\r\n") | tag("\n")).opt()
        + (multiline_char_str() | utf32_str() | utf16_str() | esc_str()).repeat(0..)
        + tag(r#"""""#).fatal(error_msg::UNCLOSED_STRING))
    .map(|strs| strs.concat())
}

fn char_str() -> Parser<impl Parse<Output = String>> {
    (is_a(|c| matches!(c, '\u{0}'..='\u{8}' | '\u{0A}'..='\u{1F}' | '\u{7F}'))
        .reject(error_msg::UNESCAPED_CTRL_CHAR)
        + none_of("\\\""))
    .repeat(1..)
    .map(|s| s.to_owned())
}

fn multiline_char_str() -> Parser<impl Parse<Output = String>> {
    // Identical to char_str, but we allow \r\n = \u{D}\u{A}
    let esc_nl = || (tag("\\\n") | tag("\\\r\n")) + one_of(" \t\r\n").repeat(0..);
    let invalid =
        |c| matches!(c, '\u{0}'..='\u{8}' | '\u{B}'..= '\u{C}' | '\u{0E}'..='\u{1F}' | '\u{7F}');

    (esc_nl().opt()
        + is_a(invalid).reject(error_msg::UNESCAPED_CTRL_CHAR)
        + none_of("\\\"").map(|s| s.to_owned()) // TODO: .collect() would be nice
        + esc_nl().opt())
    .repeat(1..)
    .map(|strs| strs.concat())
}

fn utf16_str() -> Parser<impl Parse<Output = String>> {
    let hex = is_a(|c| c.is_ascii_hexdigit())
        .repeat(4)
        .map(|s| u16::from_str_radix(s, 16).unwrap());

    (tag("\\u") + hex).repeat(1..).convert(
        |utf16| char::decode_utf16(utf16).collect(),
        error_msg::ILLEGAL_UTF16,
    )
}

fn utf32_str() -> Parser<impl Parse<Output = String>> {
    let hex = is_a(|c| c.is_ascii_hexdigit())
        .repeat(8)
        .map(|s| u32::from_str_radix(s, 16).unwrap());

    // TODO: here, we return some error which is then ignored by the convert function.
    // convert should also accept an option, or have a less strict Error bound,
    // as it is ignored anyways and we must construct an error just to satisfy the bound.
    #[derive(Debug)]
    struct DummyError;
    impl Display for DummyError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            todo!()
        }
    }
    impl Error for DummyError {}

    (tag("\\U") + hex)
        .convert(
            |utf32| char::from_u32(utf32).ok_or(DummyError),
            error_msg::ILLEGAL_UTF32,
        )
        .repeat(1..)
        .map(|s| s.into_iter().collect())
}

fn esc_str() -> Parser<impl Parse<Output = String>> {
    let esc = tag("b").val("\x08")
        | tag("t").val("\t")
        | tag("n").val("\n")
        | tag("f").val("\x0C")
        | tag("r").val("\r")
        | tag("\"").val("\"")
        | tag("\\").val("\\");

    // Would be nice to have non-static error messages (fatal accepting a closure)
    // so we can build them at runtime: e.g., mentioning the unexpected symbol

    (tag("\\") + esc.fatal(error_msg::ESCAPE_SEQUENCE))
        .repeat(1..)
        .map(|strs| strs.concat())
}

// --------------------------------
// Helper functions and definitions
// --------------------------------

fn ws() -> Matcher<impl Match> {
    one_of(" \t").repeat(0..)
}

fn ln() -> Matcher<impl Match> {
    let eol = || tag("\n") | tag("\r\n");
    let comment = || tag("#") + none_of("\r\n").repeat(1..);

    // Note: The EOF match must be separately, otherwise we get an infinite loop
    (ws() + comment().opt() + eol()).repeat(1..) + (ws() + comment().opt() + eof()).opt()
}

mod error_msg {
    pub const NO_NEWLINE: &str = "Expected newline after expression:";
    pub(super) const UNCLOSED_STRING: &str = "Missing trailing '\"' to close string literal:";
    pub(super) const ILLEGAL_UTF16: &str = "Illegal utf-16 string:";
    pub(super) const ILLEGAL_UTF32: &str = "Illegal utf-32 string:";
    pub(super) const ESCAPE_SEQUENCE: &str =
        r#"Illegal escape sequence: Only btnfr"\ are allowed:"#;
    pub(super) const UNESCAPED_CTRL_CHAR: &str = "Illegal unescaped control character:";
    pub(super) const UNKNOWN_VALUE: &str = "Failed to parse expected value:";
    pub(super) const LEADING_ZERO: &str = "Numbers cannot start with a leading zero:";
    pub(super) const NUMBER_TOO_BIG: &str = "Number does not fit in a i64 integer:";
    pub(super) const PARSE_ERROR: &str = "Internal error: failed to parse matched string:";
}
