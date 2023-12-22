use std::{error::Error, fmt::Display};

pub type ParseResult<'a, O> = Result<(O, &'a str), ParseError>;

pub type MatchResult<'a> = Result<&'a str, ParseError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Generic(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Generic(err) => err.fmt(f),
        }
    }
}

impl Error for ParseError {}
