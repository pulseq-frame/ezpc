use crate::result::{MatchResult, ParseError, ParseResult};

use super::{Match, Parse};

pub struct Repeat<T> {
    pub(crate) parser_or_matcher: T,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

pub struct Opt<T>(pub(crate) T);

pub struct ValMatch<M, T> {
    pub(crate) matcher: M,
    pub(crate) value: T,
}

pub struct MapMatch<M, F> {
    pub(crate) matcher: M,
    pub(crate) map_func: F,
}

pub struct MapParse<P, F> {
    pub(crate) parser: P,
    pub(crate) map_func: F,
}

pub struct TryMapParse<P, F> {
    pub(crate) parser: P,
    pub(crate) map_func: F,
}

// Implementations for modified Parsers

impl<T: Parse> Parse for Repeat<T> {
    type Output = Vec<T::Output>;

    fn parse<'a>(&self, mut input: &'a str) -> ParseResult<'a, Self::Output> {
        let mut items = Vec::new();

        for _ in 0..=self.end {
            if let Ok((out, rest)) = self.parser_or_matcher.parse(input) {
                items.push(out);
                input = rest;
            } else {
                break;
            }
        }

        if items.len() < self.start {
            Err(ParseError::Generic(
                "Parser didn't apply often enough".into(),
            ))
        } else {
            Ok((items, input))
        }
    }
}

impl<T: Parse> Parse for Opt<T> {
    type Output = Option<T::Output>;

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.0
            .parse(input)
            .map_or(Ok((None, input)), |(out, rest)| Ok((Some(out), rest)))
    }
}

// Implementations for modified Matchers

impl<T: Match> Match for Repeat<T> {
    fn parse<'a>(&self, mut input: &'a str) -> MatchResult<'a> {
        let mut item_count = 0;

        for _ in 0..=self.end {
            if let Ok(rest) = self.parser_or_matcher.parse(input) {
                item_count += 1;
                input = rest;
            } else {
                break;
            }
        }

        if item_count < self.start {
            Err(ParseError::Generic(
                "Parser didn't apply often enough".into(),
            ))
        } else {
            Ok(input)
        }
    }
}

impl<T: Match> Match for Opt<T> {
    fn parse<'a>(&self, input: &'a str) -> MatchResult<'a> {
        self.0.parse(input).map_or(Ok(input), |rest| Ok(rest))
    }
}

// Implementations for Mappers and Converters

impl<M: Match, T: Clone> Parse for ValMatch<M, T> {
    type Output = T;

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.matcher
            .parse(input)
            .map(|rest| (self.value.clone(), rest))
    }
}

impl<P, F, O> Parse for MapParse<P, F>
where
    P: Parse,
    F: Fn(P::Output) -> O + 'static,
{
    type Output = O;

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.parser
            .parse(input)
            .map(|(tmp, rest)| ((self.map_func)(tmp), rest))
    }
}

impl<M: Match, F, T, E> Parse for MapMatch<M, F>
where
    F: Fn(&str) -> Result<T, E>,
    E: std::error::Error + 'static,
{
    type Output = T;

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.matcher.parse(input).and_then(|rest| {
            let consumed = rest.as_ptr() as usize - input.as_ptr() as usize;
            match (self.map_func)(&input[..consumed]) {
                Ok(out) => Ok((out, rest)),
                Err(err) => Err(ParseError::Generic(err.to_string())),
            }
        })
    }
}

impl<P, F, O, E> Parse for TryMapParse<P, F>
where
    P: Parse,
    F: Fn(P::Output) -> Result<O, E> + 'static,
    E: std::error::Error + 'static,
{
    type Output = O;

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.parser
            .parse(input)
            .and_then(|(tmp, rest)| match (self.map_func)(tmp) {
                Ok(out) => Ok((out, rest)),
                Err(err) => Err(ParseError::Generic(err.to_string())),
            })
    }
}
