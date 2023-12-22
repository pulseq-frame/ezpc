use super::{Match, Parse};
use crate::result::{ParseError, ParseResult};

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

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.matcher.parse(input).and_then(|((), rest)| {
            let consumed = rest.as_ptr() as usize - input.as_ptr() as usize;
            match (self.map_func)(&input[..consumed]) {
                Ok(out) => Ok((out, rest)),
                Err(err) => Err(ParseError::Generic(err.to_string())),
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

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.matcher
            .parse(input)
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

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.parser
            .parse(input)
            .and_then(|(tmp, rest)| match (self.f)(tmp) {
                Ok(out) => Ok((out, rest)),
                Err(err) => Err(ParseError::Generic(err.to_string())),
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

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.parser
            .parse(input)
            .map(|(tmp, rest)| ((self.f)(tmp), rest))
    }
}
