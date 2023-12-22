use super::{Match, Matcher};
use crate::result::{ParseError, ParseResult};

pub struct Tag(&'static str);

pub fn tag(tag: &'static str) -> Matcher<Tag> {
    Matcher(Tag(tag))
}

impl Match for Tag {
    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, ()> {
        if input.starts_with(self.0) {
            Ok(((), &input[self.0.len()..]))
        } else {
            Err(ParseError::Generic("tag not found".into()))
        }
    }
}
