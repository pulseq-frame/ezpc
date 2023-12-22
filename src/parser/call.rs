use super::{Match, Matcher, Parse, Parser};
use crate::{input::Input, result::ParseResult};

pub struct ParseGen<O>(Box<dyn Fn() -> Box<dyn Parse<Output = O>>>);

impl<O> Parse for ParseGen<O> {
    type Output = O;

    fn parse(&self, input: Input) -> ParseResult<Self::Output> {
        (self.0)().parse(input)
    }
}

pub trait WrapParser<O> {
    fn wrap(self) -> Parser<ParseGen<O>>;
}

impl<O, P, F> WrapParser<O> for F
where
    P: Parse<Output = O> + 'static,
    F: Fn() -> Parser<P> + 'static,
{
    fn wrap(self) -> Parser<ParseGen<O>> {
        Parser(ParseGen(Box::new(move || Box::new(self().0))))
    }
}

pub struct MatchGen(Box<dyn Fn() -> Box<dyn Match>>);

impl Match for MatchGen {
    fn parse(&self, input: Input) -> ParseResult<()> {
        (self.0)().parse(input)
    }
}

pub trait WrapMatcher {
    fn wrap(self) -> Matcher<MatchGen>;
}

impl<M, F> WrapMatcher for F
where
    M: Match + 'static,
    F: Fn() -> Matcher<M> + 'static,
{
    fn wrap(self) -> Matcher<MatchGen> {
        Matcher(MatchGen(Box::new(move || Box::new(self().0))))
    }
}
