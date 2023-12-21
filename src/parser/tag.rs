use super::{Match, Matcher};
use crate::{
    input::Input,
    result::{ParseError, ParseResult},
};

pub struct Tag(&'static str);

pub fn tag(tag: &'static str) -> Matcher<Tag> {
    Matcher(Tag(tag))
}

impl Match for Tag {
    fn parse(&self, input: Input) -> ParseResult<()> {
        if input.starts_with(self.0) {
            Ok(((), input.skip(self.0.len())))
        } else {
            Err(ParseError::Generic("tag not found".into()))
        }
    }
}
