use std::{error::Error, fmt::Display};
use thiserror::Error;

pub type ParseResult<'a, O> = Result<(O, &'a str), ParseError>;

pub type MatchResult<'a> = Result<&'a str, ParseError>;

// pub enum ParseError {

// }

#[derive(Debug)]
pub enum ParseError {
    Tag(&'static str),
    OneOf(&'static str),
    NoneOf(&'static str),
    IsA(&'static str),
    List,
    Repeat { min: usize, count: usize },
    Boxed(Box<dyn Error>),
    ExpectedEof,
    Incomplete,
    RecursionDepth(usize),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Tag(tag) => write!(f, "Tag({tag:?}): not found"),
            ParseError::OneOf(bag) => write!(f, "OneOf({bag:?}): nothing matched"),
            ParseError::NoneOf(bag) => write!(f, "NoneOf({bag:?}): something matched"),
            ParseError::IsA(descr) => write!(f, "IsA({descr}): Predicate didn't apply"),
            ParseError::List => write!(f, "List: element parser didn't apply"),
            ParseError::Repeat { min, count } => {
                write!(f, "Repeat: Parser applied {count} times, minimum was {min}")
            }
            ParseError::Boxed(err) => err.fmt(f),
            ParseError::ExpectedEof => write!(f, "ExpectedEof: Expected end of input"),
            ParseError::Incomplete => write!(f, "Incomplete: Didn't parse until EOF"),
            ParseError::RecursionDepth(depth) => {
                write!(f, "RecursionDepth({depth}): Nested too deep")
            }
        }
    }
}

impl Error for ParseError {}
