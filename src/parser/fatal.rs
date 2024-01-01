use std::{fmt::Display, ops::Not};

use super::{Match, Matcher, Parse, Parser};
use crate::result::{MatchResult, ParseError, ParseResult};

pub struct Fatal<T>(T);

impl<P: Parse> Not for Parser<P> {
    type Output = Parser<Fatal<P>>;

    fn not(self) -> Self::Output {
        Parser(Fatal(self.0))
    }
}

impl<M: Match> Not for Matcher<M> {
    type Output = Matcher<Fatal<M>>;

    fn not(self) -> Self::Output {
        Matcher(Fatal(self.0))
    }
}

impl<T: Display> Display for Fatal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "!{}", self.0)
    }
}

impl<P: Parse> Parse for Fatal<P> {
    type Output = P::Output;

    fn apply<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.0.apply(input).map_err(|err| match err {
            ParseError::Mismatch(_) => ParseError::Fatal {
                expected: format!("{}", self.0),
                at: esc_trunc(input),
            },
            _ => err,
        })
    }
}

impl<M: Match> Match for Fatal<M> {
    fn apply<'a>(&self, input: &'a str) -> MatchResult<'a> {
        self.0.apply(input).map_err(|err| match err {
            ParseError::Mismatch(_) => ParseError::Fatal {
                expected: format!("{}", self.0),
                at: esc_trunc(input),
            },
            _ => err,
        })
    }
}

pub fn esc_trunc(mut input: &str) -> String {
    if let Some(pos) = input.find('\r').or(input.find('\n')) {
        input = &input[pos + 1..];
    }
    if input.len() <= 20 {
        input.to_owned()
    } else {
        format!("{}...", &input[1..16])
    }
}
