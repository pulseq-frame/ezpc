use std::{error::Error, fmt::Display};

use crate::input::Input;

pub type ParseResult<O> = Result<(O, Input), ParseError>;

#[derive(Debug)]
pub enum ParseError {
    Generic(Box<dyn Error>),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Generic(err) => err.fmt(f),
        }
    }
}

impl Error for ParseError {}
