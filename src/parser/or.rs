use std::ops::BitOr;

use super::{Match, Matcher, Parse, Parser};
use crate::{input::Input, result::ParseResult};

pub struct OrPP<P1: Parse, P2: Parse>(P1, P2);
pub struct OrMM<M1: Match, M2: Match>(M1, M2);

impl<P1, P2> Parse for OrPP<P1, P2>
where
    P1: Parse,
    P2: Parse<Output = P1::Output>,
{
    type Output = P1::Output;

    fn parse(&self, input: Input) -> ParseResult<Self::Output> {
        match self.0.parse(input.clone()) {
            Ok((out, rest)) => Ok((out, rest)),
            Err(_) => self.1.parse(input),
        }
    }
}

impl<M1: Match, M2: Match> Match for OrMM<M1, M2> {
    fn parse(&self, input: Input) -> ParseResult<()> {
        match self.0.parse(input.clone()) {
            Ok(((), rest)) => Ok(((), rest)),
            Err(_) => self.1.parse(input),
        }
    }
}

// Overload std::ops::BitOr to create this parser combinator

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
