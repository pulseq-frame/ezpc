use super::{Match, Matcher};
use crate::result::{MatchResult, ParseError};

pub struct NoneOf(&'static str);

pub fn none_of(bag: &'static str) -> Matcher<NoneOf> {
    Matcher(NoneOf(bag))
}

impl Match for NoneOf {
    fn parse<'a>(&self, input: &'a str) -> MatchResult<'a> {
        if let Some((c, rest)) = pop_char(input) {
            if self.0.contains(c) {
                Err(ParseError::Generic(
                    "one of the expected characters found".into(),
                ))
            } else {
                Ok(rest)
            }
        } else {
            Err(ParseError::Generic("Input is empty".into()))
        }
    }
}

fn pop_char(s: &str) -> Option<(char, &str)> {
    s.chars().next().map(|c| (c, &s[c.len_utf8()..]))
}
