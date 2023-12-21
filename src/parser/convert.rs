use super::{Match, Parse};
use crate::{
    input::Input,
    result::{ParseError, ParseResult},
};

pub struct MapMatch<M, F> {
    pub(crate) matcher: M,
    pub(crate) map_func: F,
}

impl<M: Match, F, T, E> Parse for MapMatch<M, F>
where
    F: Fn(&str) -> Result<T, E>,
    E: std::error::Error + 'static,
{
    type Output = T;

    fn parse(&self, input: Input) -> ParseResult<Self::Output> {
        self.matcher.parse(input.clone()).and_then(|((), rest)| {
            match (self.map_func)(rest.get_consumed(&input)) {
                Ok(out) => Ok((out, rest)),
                Err(err) => Err(ParseError::Generic(err.into())),
            }
        })
    }
}
