pub mod combinators;
pub mod combine_ops;
pub mod matchers;
pub mod modifiers;
pub mod wrap;

use crate::{
    range::RangeArgument,
    result::{MatchResult, ParseError, ParseResult},
};

use self::modifiers::{
    MapMatch, MapParse, Opt, Repeat, TryMapMatch, TryMapParse, ValMatch, ValParse,
};

pub trait Parse {
    type Output;
    fn apply<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output>;
}

pub struct Parser<T: Parse>(T);

impl<P: Parse> Parser<P> {
    pub fn repeat<R: RangeArgument>(self, range: R) -> Parser<Repeat<P>> {
        Parser(Repeat {
            parser_or_matcher: self.0,
            start: range.start(),
            end: range.end(),
        })
    }

    pub fn opt(self) -> Parser<Opt<P>> {
        Parser(Opt(self.0))
    }

    pub fn val<O: Clone>(self, value: O) -> Parser<ValParse<P, O>> {
        Parser(ValParse {
            parser: self.0,
            value,
        })
    }

    pub fn map<F, O>(self, f: F) -> Parser<MapParse<P, F>>
    where
        F: Fn(P::Output) -> O + 'static,
    {
        Parser(MapParse {
            parser: self.0,
            map_func: f,
        })
    }

    pub fn try_map<F, O, E>(self, f: F) -> Parser<TryMapParse<P, F>>
    where
        F: Fn(P::Output) -> Result<O, E> + 'static,
        E: std::error::Error + 'static,
    {
        Parser(TryMapParse {
            parser: self.0,
            map_func: f,
        })
    }

    pub fn parse(&self, source: &str) -> Result<P::Output, ParseError> {
        self.0.apply(source.into()).and_then(|(out, rest)| {
            if rest.is_empty() {
                Ok(out)
            } else {
                Err(ParseError::Generic("Didn't parse to EOF".into()))
            }
        })
    }
}

pub trait Match {
    fn apply<'a>(&self, input: &'a str) -> MatchResult<'a>;
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

    pub fn map<F, O>(self, map_func: F) -> Parser<MapMatch<M, F>>
    where
        F: Fn(&str) -> O + 'static,
    {
        Parser(MapMatch {
            matcher: self.0,
            map_func,
        })
    }

    pub fn try_map<F, T, E>(self, map_func: F) -> Parser<TryMapMatch<M, F>>
    where
        F: Fn(&str) -> Result<T, E> + 'static,
        E: std::error::Error + 'static,
    {
        Parser(TryMapMatch {
            matcher: self.0,
            map_func,
        })
    }
}
