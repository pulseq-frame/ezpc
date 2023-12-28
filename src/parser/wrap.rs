use super::{Match, Matcher, Parse, Parser};
use crate::result::{MatchResult, ParseError, ParseResult};

pub struct DynParse<O: 'static> {
    max_depth: usize,
    parser: Box<dyn Fn() -> Box<dyn Parse<Output = O>>>,
}
pub trait WrapParser<O: 'static> {
    fn wrap(self, max_depth: usize) -> Parser<DynParse<O>>;
}

pub struct DynMatch {
    max_depth: usize,
    matcher: Box<dyn Fn() -> Box<dyn Match>>,
}
pub trait WrapMatcher {
    fn wrap(self, max_depth: usize) -> Matcher<DynMatch>;
}

// Implement the wrapping itself

impl<O: 'static, P, F> WrapParser<O> for F
where
    P: Parse<Output = O> + 'static,
    F: Fn() -> Parser<P> + 'static,
{
    fn wrap(self, max_depth: usize) -> Parser<DynParse<O>> {
        Parser(DynParse {
            max_depth,
            parser: Box::new(move || Box::new(self().0)),
        })
    }
}

impl<M, F> WrapMatcher for F
where
    M: Match + 'static,
    F: Fn() -> Matcher<M> + 'static,
{
    fn wrap(self, max_depth: usize) -> Matcher<DynMatch> {
        Matcher(DynMatch {
            max_depth,
            matcher: Box::new(move || Box::new(self().0)),
        })
    }
}

// Implement Parse and Match traits for the wrapped parsers / matchers

impl<O: 'static> Parse for DynParse<O> {
    type Output = O;

    fn apply<'a>(&self, input: &'a str, depth: usize) -> ParseResult<'a, Self::Output> {
        if depth > self.max_depth {
            log::trace!(
                "Cancel parsing: Maximum recursion depth ({}) reached",
                self.max_depth
            );
            return Err(ParseError::RecursionDepth(depth));
        }
        (self.parser)().apply(input, depth + 1)
    }
}

impl Match for DynMatch {
    fn apply<'a>(&self, input: &'a str, depth: usize) -> MatchResult<'a> {
        if depth > self.max_depth {
            log::trace!(
                "Cancel matching: Maximum recursion depth ({}) reached",
                self.max_depth
            );
            return Err(ParseError::RecursionDepth(depth));
        }
        (self.matcher)().apply(input, depth + 1)
    }
}
