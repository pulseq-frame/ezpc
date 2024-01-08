use std::fmt::Display;

use super::{Match, Matcher};
use crate::result::{MatchResult, RawEzpcError};

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
            Err(RawEzpcError::Mismatch {
                pos: input.as_ptr(),
            })
        }
    }
}

impl Match for Tag {
    fn apply<'a>(&self, input: &'a str) -> MatchResult<'a> {
        if let Some(rest) = input.strip_prefix(self.0) {
            Ok(rest)
        } else {
            Err(RawEzpcError::Mismatch {
                pos: input.as_ptr(),
            })
        }
    }
}

impl Match for OneOf {
    fn apply<'a>(&self, input: &'a str) -> MatchResult<'a> {
        if let Some((c, rest)) = pop_char(input) {
            if self.0.contains(c) {
                return Ok(rest);
            }
        }
        Err(RawEzpcError::Mismatch {
            pos: input.as_ptr(),
        })
    }
}

impl Match for NoneOf {
    fn apply<'a>(&self, input: &'a str) -> MatchResult<'a> {
        if let Some((c, rest)) = pop_char(input) {
            if !self.0.contains(c) {
                return Ok(rest);
            }
        }
        Err(RawEzpcError::Mismatch {
            pos: input.as_ptr(),
        })
    }
}

impl<F> Match for IsA<F>
where
    F: Fn(char) -> bool,
{
    fn apply<'a>(&self, input: &'a str) -> MatchResult<'a> {
        if let Some((c, rest)) = pop_char(input) {
            if (self.0)(c) {
                return Ok(rest);
            }
        }
        Err(RawEzpcError::Mismatch {
            pos: input.as_ptr(),
        })
    }
}

// Display impls for all matchers

impl Display for Eof {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EOF")
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Display for OneOf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}]", self.0)
    }
}

impl Display for NoneOf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[!{:?}]", self.0)
    }
}

impl<F> Display for IsA<F>
where
    F: Fn(char) -> bool,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IsA(...)")
    }
}

/// Helper function that splits a string into the first char and rest
fn pop_char(s: &str) -> Option<(char, &str)> {
    s.chars().next().map(|c| (c, &s[c.len_utf8()..]))
}
