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

pub struct IsA<F> {
    predicate: F,
    description: &'static str,
}
pub fn is_a<F>(description: &'static str, predicate: F) -> Matcher<IsA<F>>
where
    F: Fn(char) -> bool,
{
    Matcher(IsA {
        predicate,
        description,
    })
}

// All the Match implementations for the Matchers above

impl Match for Eof {
    fn apply<'a>(&self, input: &'a str, _depth: usize) -> MatchResult<'a> {
        if input.is_empty() {
            log::trace!("MATCH! {} - Eof", log_input(input));
            Ok(input)
        } else {
            log::trace!("failed {} - Eof", log_input(input));
            Err(ParseError::Incomplete)
        }
    }
}

impl Match for Tag {
    fn apply<'a>(&self, input: &'a str, _depth: usize) -> MatchResult<'a> {
        if let Some(rest) = input.strip_prefix(self.0) {
            log::trace!("MATCH! {} - Tag({:?})", log_input(input), self.0);
            Ok(rest)
        } else {
            log::trace!("failed {} - Tag({:?})", log_input(input), self.0);
            Err(ParseError::Tag(self.0))
        }
    }
}

impl Match for OneOf {
    fn apply<'a>(&self, input: &'a str, _depth: usize) -> MatchResult<'a> {
        if let Some((c, rest)) = pop_char(input) {
            if self.0.contains(c) {
                log::trace!("MATCH! {} - OneOf({:?})", log_input(input), self.0);
                return Ok(rest);
            }
        }
        log::trace!("failed {} - OneOf({:?})", log_input(input), self.0);
        Err(ParseError::OneOf(self.0))
    }
}

impl Match for NoneOf {
    fn apply<'a>(&self, input: &'a str, _depth: usize) -> MatchResult<'a> {
        if let Some((c, rest)) = pop_char(input) {
            if !self.0.contains(c) {
                log::trace!("MATCH! {} - NoneOf({:?})", log_input(input), self.0);
                return Ok(rest);
            }
        }
        log::trace!("failed {} - NoneOf({:?})", log_input(input), self.0);
        Err(ParseError::NoneOf(self.0))
    }
}

impl<F> Match for IsA<F>
where
    F: Fn(char) -> bool,
{
    fn apply<'a>(&self, input: &'a str, _depth: usize) -> MatchResult<'a> {
        if let Some((c, rest)) = pop_char(input) {
            if (self.predicate)(c) {
                log::trace!("MATCH! {} - IsA", log_input(input));
                return Ok(rest);
            }
        }
        log::trace!("failed {} - IsA", log_input(input));
        Err(ParseError::IsA(self.description))
    }
}

/// Helper function that splits a string into the first char and rest
fn pop_char(s: &str) -> Option<(char, &str)> {
    s.chars().next().map(|c| (c, &s[c.len_utf8()..]))
}

/// Helper function for formatting the the input string on logging
fn log_input(input: &str) -> String {
    let tmp: String = input.escape_debug().take(15).collect();
    if tmp.len() <= 14 {
        format!("\"{tmp}\"")
    } else {
        format!("\"{tmp:.11}...\"")
    }
}
