use std::ops::Add;
use std::ops::BitOr;

use super::{Match, Matcher, Parse, Parser};
use crate::result::{MatchResult, ParseResult};

// Sequence of parsers or matchers, produced by adding (+) them
pub struct AndPP<P1: Parse, P2: Parse>(P1, P2);
pub struct AndPM<P1: Parse, M2: Match>(P1, M2);
pub struct AndMP<M1: Match, P2: Parse>(M1, P2);
pub struct AndMM<M1: Match, M2: Match>(M1, M2);

// Ordered choice of parsers or matchers, produced by bit or'ing (|) them
pub struct OrPP<P1: Parse, P2: Parse>(P1, P2);
pub struct OrMM<M1: Match, M2: Match>(M1, M2);

// Implement Add operator for sequence

impl<P1: Parse, P2: Parse> Add<Parser<P2>> for Parser<P1> {
    type Output = Parser<AndPP<P1, P2>>;

    fn add(self, rhs: Parser<P2>) -> Self::Output {
        Parser(AndPP(self.0, rhs.0))
    }
}

impl<P1: Parse, M2: Match> Add<Matcher<M2>> for Parser<P1> {
    type Output = Parser<AndPM<P1, M2>>;

    fn add(self, rhs: Matcher<M2>) -> Self::Output {
        Parser(AndPM(self.0, rhs.0))
    }
}

impl<M1: Match, P2: Parse> Add<Parser<P2>> for Matcher<M1> {
    type Output = Parser<AndMP<M1, P2>>;

    fn add(self, rhs: Parser<P2>) -> Self::Output {
        Parser(AndMP(self.0, rhs.0))
    }
}

impl<M1: Match, M2: Match> Add<Matcher<M2>> for Matcher<M1> {
    type Output = Matcher<AndMM<M1, M2>>;

    fn add(self, rhs: Matcher<M2>) -> Self::Output {
        Matcher(AndMM(self.0, rhs.0))
    }
}

// Implement BitOr operator for ordered choice

impl<P1, P2> BitOr<Parser<P2>> for Parser<P1>
where
    P1: Parse,
    P2: Parse<Output = P1::Output>,
{
    type Output = Parser<OrPP<P1, P2>>;

    fn bitor(self, rhs: Parser<P2>) -> Self::Output {
        Parser(OrPP(self.0, rhs.0))
    }
}

impl<M1: Match, M2: Match> BitOr<Matcher<M2>> for Matcher<M1> {
    type Output = Matcher<OrMM<M1, M2>>;

    fn bitor(self, rhs: Matcher<M2>) -> Self::Output {
        Matcher(OrMM(self.0, rhs.0))
    }
}

// Implement Parse and Match for Add (Sequence)

impl<P1: Parse, P2: Parse> Parse for AndPP<P1, P2> {
    type Output = (P1::Output, P2::Output);

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.0
            .parse(input)
            .and_then(|(out1, rest)| self.1.parse(rest).map(|(out2, rest)| ((out1, out2), rest)))
    }
}

impl<P1: Parse, M2: Match> Parse for AndPM<P1, M2> {
    type Output = P1::Output;

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.0
            .parse(input)
            .and_then(|(out, rest)| self.1.parse(rest).map(|rest| (out, rest)))
    }
}

impl<M1: Match, P2: Parse> Parse for AndMP<M1, P2> {
    type Output = P2::Output;

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.0
            .parse(input)
            .and_then(|rest| self.1.parse(rest).map(|(out, rest)| (out, rest)))
    }
}

impl<M1: Match, M2: Match> Match for AndMM<M1, M2> {
    fn parse<'a>(&self, input: &'a str) -> MatchResult<'a> {
        self.0
            .parse(input)
            .and_then(|rest| self.1.parse(rest).map(|rest| rest))
    }
}

// Implement Parse and Match for Or (Ordered choice)

impl<P1, P2> Parse for OrPP<P1, P2>
where
    P1: Parse,
    P2: Parse<Output = P1::Output>,
{
    type Output = P1::Output;

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        match self.0.parse(input) {
            Ok((out, rest)) => Ok((out, rest)),
            Err(_) => self.1.parse(input),
        }
    }
}

impl<M1: Match, M2: Match> Match for OrMM<M1, M2> {
    fn parse<'a>(&self, input: &'a str) -> MatchResult<'a> {
        match self.0.parse(input) {
            Ok(rest) => Ok(rest),
            Err(_) => self.1.parse(input),
        }
    }
}
