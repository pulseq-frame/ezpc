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

pub struct MapVal<M, O> {
    pub(crate) matcher: M,
    pub(crate) value: O,
}

impl<M: Match, O: Clone> Parse for MapVal<M, O> {
    type Output = O;

    fn parse(&self, input: Input) -> ParseResult<Self::Output> {
        self.matcher
            .parse(input.clone())
            .map(|((), rest)| (self.value.clone(), rest))
    }
}

pub struct Convert<P, F, O, E>
where
    P: Parse,
    F: Fn(P::Output) -> Result<O, E> + 'static,
{
    pub(crate) parser: P,
    pub(crate) f: F,
}

impl<P, F, O, E> Parse for Convert<P, F, O, E>
where
    P: Parse,
    F: Fn(P::Output) -> Result<O, E> + 'static,
    E: std::error::Error + 'static,
{
    type Output = O;

    fn parse(&self, input: Input) -> ParseResult<Self::Output> {
        self.parser
            .parse(input)
            .and_then(|(tmp, rest)| match (self.f)(tmp) {
                Ok(out) => Ok((out, rest)),
                Err(err) => Err(ParseError::Generic(err.into())),
            })
    }
}

pub struct Map<P, F, O>
where
    P: Parse,
    F: Fn(P::Output) -> O + 'static,
{
    pub(crate) parser: P,
    pub(crate) f: F,
}

impl<P, F, O> Parse for Map<P, F, O>
where
    P: Parse,
    F: Fn(P::Output) -> O + 'static,
{
    type Output = O;

    fn parse(&self, input: Input) -> ParseResult<Self::Output> {
        self.parser
            .parse(input)
            .map(|(tmp, rest)| ((self.f)(tmp), rest))
    }
}
