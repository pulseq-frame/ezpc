use super::{Match, Matcher, Parse, Parser};
use crate::result::{MatchResult, ParseResult};

pub struct DynParse<O: 'static>(Box<dyn Fn() -> Box<dyn Parse<Output = O>>>);
pub trait WrapParser<O: 'static> {
    fn wrap(self) -> Parser<DynParse<O>>;
}

pub struct DynMatch(Box<dyn Fn() -> Box<dyn Match>>);
pub trait WrapMatcher {
    fn wrap(self) -> Matcher<DynMatch>;
}

// Implement the wrapping itself

impl<O: 'static, P, F> WrapParser<O> for F
where
    P: Parse<Output = O> + 'static,
    F: Fn() -> Parser<P> + 'static,
{
    fn wrap(self) -> Parser<DynParse<O>> {
        Parser(DynParse(Box::new(move || Box::new(self().0))))
    }
}

impl<M, F> WrapMatcher for F
where
    M: Match + 'static,
    F: Fn() -> Matcher<M> + 'static,
{
    fn wrap(self) -> Matcher<DynMatch> {
        Matcher(DynMatch(Box::new(move || Box::new(self().0))))
    }
}

// Implement Parse and Match traits for the wrapped parsers / matchers

impl Match for DynMatch {
    fn parse<'a>(&self, input: &'a str) -> MatchResult<'a> {
        (self.0)().parse(input)
    }
}

impl<O: 'static> Parse for DynParse<O> {
    type Output = O;

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        (self.0)().parse(input)
    }
}
