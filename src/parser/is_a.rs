use super::{Match, Matcher};
use crate::result::{ParseError, ParseResult};

pub struct IsA<F>(F);

pub fn is_a<F>(predicate: F) -> Matcher<IsA<F>>
where
    F: Fn(char) -> bool,
{
    Matcher(IsA(predicate))
}

impl<F> Match for IsA<F>
where
    F: Fn(char) -> bool,
{
    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, ()> {
        if let Some((c, input)) = pop_char(input) {
            if (self.0)(c) {
                Ok(((), input))
            } else {
                Err(ParseError::Generic(
                    "Predicate didn't apply".into(),
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
