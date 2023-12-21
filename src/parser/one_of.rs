use super::{Match, Matcher};
use crate::{
    input::Input,
    result::{ParseError, ParseResult},
};

pub struct OneOf(&'static str);

pub fn one_of(bag: &'static str) -> Matcher<OneOf> {
    Matcher(OneOf(bag))
}

impl Match for OneOf {
    fn parse(&self, input: Input) -> ParseResult<()> {
        if let Some((c, input)) = input.pop_char() {
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
