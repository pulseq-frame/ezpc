use super::{Match, Matcher};
use crate::result::{ParseError, ParseResult};

pub struct OneOf(&'static str);

pub fn one_of(bag: &'static str) -> Matcher<OneOf> {
    Matcher(OneOf(bag))
}

impl Match for OneOf {
    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, ()> {
        if let Some((c, input)) = pop_char(input) {
            if self.0.contains(c) {
                Ok(((), input))
            } else {
                Err(ParseError::Generic(
                    "none of the expected characters found".into(),
                ))
            }
        } else {
            Err(ParseError::Generic("Input is empty".into()))
        }
    }
}

fn pop_char(s: &str) -> Option<(char, &str)> {
    s.chars().next().map(|c| (c, &s[c.len_utf8()..]))
}
