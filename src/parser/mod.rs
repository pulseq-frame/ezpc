pub mod combinators;
pub mod combine_ops;
pub mod matchers;
pub mod modifiers;
pub mod wrap;

use crate::{
    range::RangeArgument,
    result::{MatchResult, ParseError, ParseResult},
};

use self::modifiers::{MapMatch, MapParse, Opt, Repeat, TryMapParse, ValMatch};

pub trait Parse {
    type Output;
    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output>;
}

pub struct Parser<T: Parse>(T);

impl<T: Parse> Parser<T> {
    pub fn repeat<R: RangeArgument>(self, range: R) -> Parser<Repeat<T>> {
        Parser(Repeat {
            parser_or_matcher: self.0,
            start: range.start(),
            end: range.end(),
        })
    }

    pub fn opt(self) -> Parser<Opt<T>> {
        Parser(Opt(self.0))
    }

    pub fn map<F, O>(self, f: F) -> Parser<MapParse<T, F>>
    where
        F: Fn(T::Output) -> O + 'static,
    {
        Parser(MapParse {
            parser: self.0,
            map_func: f,
        })
    }

    pub fn try_map<F, O, E>(self, f: F) -> Parser<TryMapParse<T, F>>
    where
        F: Fn(T::Output) -> Result<O, E> + 'static,
        E: std::error::Error + 'static,
    {
        Parser(TryMapParse {
            parser: self.0,
            map_func: f,
        })
    }

    pub fn parse(&self, source: &str) -> Result<T::Output, ParseError> {
        self.0.parse(source.into()).and_then(|(out, rest)| {
            if rest.is_empty() {
                Ok(out)
            } else {
                Err(ParseError::Generic("Didn't parse to EOF".into()))
            }
        })
    }
}

pub trait Match {
    fn parse<'a>(&self, input: &'a str) -> MatchResult<'a>;
}

pub struct Matcher<M: Match>(M);

impl<M: Match> Matcher<M> {
    pub fn repeat<R: RangeArgument>(self, range: R) -> Matcher<Repeat<M>> {
        Matcher(Repeat {
            parser_or_matcher: self.0,
            start: range.start(),
            end: range.end(),
        })
    }

    pub fn opt(self) -> Matcher<Opt<M>> {
        Matcher(Opt(self.0))
    }

    pub fn val<O: Clone>(self, value: O) -> Parser<ValMatch<M, O>> {
        Parser(ValMatch {
            matcher: self.0,
            value,
        })
    }

    pub fn try_map<F, T, E>(self, f: F) -> Parser<MapMatch<M, F>>
    where
        F: Fn(&str) -> Result<T, E>,
        E: std::error::Error + 'static,
    {
        Parser(MapMatch {
            matcher: self.0,
            map_func: f,
        })
    }
}
