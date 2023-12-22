use super::{Match, Matcher};
use crate::result::{ParseError, ParseResult};

pub struct NoneOf(&'static str);

pub fn none_of(bag: &'static str) -> Matcher<NoneOf> {
    Matcher(NoneOf(bag))
}

impl Match for NoneOf {
    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, ()> {
        if let Some((c, input)) = pop_char(input) {
            if self.0.contains(c) {
                Err(ParseError::Generic(
                    "one of the expected characters found".into(),
                ))
            } else {
                Ok(((), input))
            }
        } else {
            Err(ParseError::Generic("Input is empty".into()))
        }
    }
}

fn pop_char(s: &str) -> Option<(char, &str)> {
    s.chars().next().map(|c| (c, &s[c.len_utf8()..]))
}
