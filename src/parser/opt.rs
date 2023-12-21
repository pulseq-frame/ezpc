use crate::{input::Input, result::ParseResult};

use super::{Parse, Match};

pub struct Opt<T>(pub(crate) T);

impl<T: Parse> Parse for Opt<T> {
    type Output = Option<T::Output>;

    fn parse(&self, input: Input) -> ParseResult<Self::Output> {
        self.0
            .parse(input.clone())
            .map_or(Ok((None, input)), |(out, rest)| Ok((Some(out), rest)))
    }
}

impl<T: Match> Match for Opt<T> {
    fn parse(&self, input: Input) -> ParseResult<()> {
        self.0
            .parse(input.clone())
            .map_or(Ok(((), input)), |((), rest)| Ok(((), rest)))
    }
}
