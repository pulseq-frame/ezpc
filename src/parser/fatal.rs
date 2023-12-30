use std::{fmt::Display, ops::Not};

use super::{Match, Matcher, Parse, Parser};
use crate::result::{MatchResult, ParseError, ParseResult};

pub struct FatalP<P: Parse>(P);

impl<P: Parse> Not for Parser<P> {
    type Output = Parser<FatalP<P>>;

    fn not(self) -> Self::Output {
        Parser(FatalP(self.0))
    }
}

impl<P: Parse> Parse for FatalP<P> {
    type Output = P::Output;

    fn apply<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.0.apply(input).map_err(|err| match err {
            ParseError::Mismatch(_) => ParseError::Fatal(format!("{}", self.0)),
            ParseError::Fatal(_) => err,
            ParseError::RecursionDepth(_) => err,
        })
    }
}

impl<P: Parse> Display for FatalP<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fatal({})", self.0)
    }
}

pub struct FatalM<M: Match>(M);

impl<M: Match> Not for Matcher<M> {
    type Output = Matcher<FatalM<M>>;

    fn not(self) -> Self::Output {
        Matcher(FatalM(self.0))
    }
}

impl<M: Match> Match for FatalM<M> {
    fn apply<'a>(&self, input: &'a str) -> MatchResult<'a> {
        self.0.apply(input).map_err(|err| match err {
            ParseError::Mismatch(_) => ParseError::Fatal(format!("{}", self.0)),
            ParseError::Fatal(_) => err,
            ParseError::RecursionDepth(_) => err,
        })
    }
}

impl<M: Match> Display for FatalM<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fatal({})", self.0)
    }
}
