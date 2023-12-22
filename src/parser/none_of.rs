use super::{Match, Matcher};
use crate::{
    input::Input,
    result::{ParseError, ParseResult},
};

pub struct NoneOf(&'static str);

pub fn none_of(bag: &'static str) -> Matcher<NoneOf> {
    Matcher(NoneOf(bag))
}

impl Match for NoneOf {
    fn parse(&self, input: Input) -> ParseResult<()> {
        if let Some((c, input)) = input.pop_char() {
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
