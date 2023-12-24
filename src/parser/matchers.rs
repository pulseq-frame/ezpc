use super::{Match, Matcher};
use crate::result::{MatchResult, ParseError};

pub struct Eof;
pub fn eof() -> Matcher<Eof> {
    Matcher(Eof)
}

pub struct Tag(&'static str);
pub fn tag(tag: &'static str) -> Matcher<Tag> {
    Matcher(Tag(tag))
}

pub struct OneOf(&'static str);
pub fn one_of(bag: &'static str) -> Matcher<OneOf> {
    Matcher(OneOf(bag))
}

pub struct NoneOf(&'static str);
pub fn none_of(bag: &'static str) -> Matcher<NoneOf> {
    Matcher(NoneOf(bag))
}

pub struct IsA<F>(F);
pub fn is_a<F>(predicate: F) -> Matcher<IsA<F>>
where
    F: Fn(char) -> bool,
{
    Matcher(IsA(predicate))
}

// All the Match implementations for the Matchers above

impl Match for Eof {
    fn apply<'a>(&self, input: &'a str) -> MatchResult<'a> {
        if input.is_empty() {
            Ok(input)
        } else {
            Err(ParseError::Incomplete)
        }
    }
}

impl Match for Tag {
    fn apply<'a>(&self, input: &'a str) -> MatchResult<'a> {
        if let Some(rest) = input.strip_prefix(self.0) {
            Ok(rest)
        } else {
            Err(ParseError::Tag(self.0))
        }
    }
}

impl Match for OneOf {
    fn apply<'a>(&self, input: &'a str) -> MatchResult<'a> {
        if let Some((c, input)) = pop_char(input) {
            if self.0.contains(c) {
                Ok(input)
            } else {
                Err(ParseError::OneOf(self.0))
            }
        } else {
            Err(ParseError::UnexpectedEof)
        }
    }
}

impl Match for NoneOf {
    fn apply<'a>(&self, input: &'a str) -> MatchResult<'a> {
        if let Some((c, rest)) = pop_char(input) {
            if self.0.contains(c) {
                Err(ParseError::NoneOf(self.0))
            } else {
                Ok(rest)
            }
        } else {
            Err(ParseError::UnexpectedEof)
        }
    }
}

impl<F> Match for IsA<F>
where
    F: Fn(char) -> bool,
{
    fn apply<'a>(&self, input: &'a str) -> MatchResult<'a> {
        if let Some((c, rest)) = pop_char(input) {
            if (self.0)(c) {
                Ok(rest)
            } else {
                Err(ParseError::IsA)
            }
        } else {
            Err(ParseError::UnexpectedEof)
        }
    }
}

/// Helper function that splits a string into the first char and rest
fn pop_char(s: &str) -> Option<(char, &str)> {
    s.chars().next().map(|c| (c, &s[c.len_utf8()..]))
}
