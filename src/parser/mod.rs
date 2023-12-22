pub mod add;
pub mod call;
pub mod convert;
pub mod list;
pub mod none_of;
pub mod one_of;
pub mod is_a;
pub mod opt;
pub mod or;
pub mod repeat;
pub mod tag;

use crate::{
    range::RangeArgument,
    result::{ParseError, ParseResult},
};
use opt::Opt;

use self::{
    convert::{Convert, Map, MapMatch, MapVal},
    repeat::Repeat,
};

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

    pub fn convert<F, O, E>(self, f: F) -> Parser<Convert<T, F, O, E>>
    where
        F: Fn(T::Output) -> Result<O, E> + 'static,
        E: std::error::Error + 'static,
    {
        Parser(Convert { parser: self.0, f })
    }

    pub fn map<F, O>(self, f: F) -> Parser<Map<T, F, O>>
    where
        F: Fn(T::Output) -> O + 'static,
    {
        Parser(Map { parser: self.0, f })
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

// TODO: for cleaner code, could return Result<Input, ParseError>, because
// this would remove a bunch of () from the code. Also, rename parse to match
pub trait Match {
    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, ()>;
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

    pub fn val<O: Clone>(self, value: O) -> Parser<MapVal<M, O>> {
        Parser(MapVal {
            matcher: self.0,
            value,
        })
    }

    pub fn convert<F, T, E>(self, f: F) -> Parser<MapMatch<M, F>>
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
