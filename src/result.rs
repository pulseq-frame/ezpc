use std::error::Error;
use thiserror::Error;

pub type ParseResult<'a, O> = Result<(O, &'a str), ParseError>;

pub type MatchResult<'a> = Result<&'a str, ParseError>;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Matcher didn't apply: {0}")]
    Mismatch(MatcherError),
    #[error("Expected: {expected}, found: '{at}'")]
    Fatal { expected: String, at: String },
    #[error("RecursionDepth({0}): Nested too deep")]
    RecursionDepth(usize),
}

#[derive(Debug, Error)]
pub enum MatcherError {
    #[error("Tag({0:?}): not found")]
    Tag(&'static str),
    #[error("OneOf({0:?}): nothing matched")]
    OneOf(&'static str),
    #[error("NoneOf({0:?}): something matched")]
    NoneOf(&'static str),
    #[error("IsA({0}): Predicate didn't apply")]
    IsA(&'static str),
    #[error("List: element parser didn't apply")]
    List,
    #[error("Repeat: Parser applied {count} times, minimum was {min}")]
    Repeat { min: usize, count: usize },
    #[error("{0}")]
    Boxed(Box<dyn Error>),
    #[error("Eof: Expected end of input")]
    Eof,
}

// TODO: Repeat and List should contain the original error code
