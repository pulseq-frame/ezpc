use super::{Match, Matcher, Parse, Parser};
use crate::result::{MatchResult, ParseResult};

pub struct ParseGen<O: 'static>(Box<dyn Fn() -> Box<dyn Parse<Output = O>>>);

impl<O: 'static> Parse for ParseGen<O> {
    type Output = O;

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        (self.0)().parse(input)
    }
}

pub trait WrapParser<O: 'static> {
    fn wrap(self) -> Parser<ParseGen<O>>;
}

impl<O: 'static, P, F> WrapParser<O> for F
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
    fn parse<'a>(&self, input: &'a str) -> MatchResult<'a> {
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
