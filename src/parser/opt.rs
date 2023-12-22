use crate::result::{MatchResult, ParseResult};

use super::{Match, Parse};

pub struct Opt<T>(pub(crate) T);

impl<T: Parse> Parse for Opt<T> {
    type Output = Option<T::Output>;

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.0
            .parse(input)
            .map_or(Ok((None, input)), |(out, rest)| Ok((Some(out), rest)))
    }
}

impl<T: Match> Match for Opt<T> {
    fn parse<'a>(&self, input: &'a str) -> MatchResult<'a> {
        self.0.parse(input).map_or(Ok(input), |rest| Ok(rest))
    }
}
