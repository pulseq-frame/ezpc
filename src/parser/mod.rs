pub mod add;
pub mod convert;
pub mod one_of;
pub mod opt;
pub mod or;
pub mod repeat;
pub mod tag;

use crate::{
    input::Input,
    range::RangeArgument,
    result::{ParseError, ParseResult},
};
use opt::Opt;

use self::{convert::MapMatch, repeat::Repeat};

pub trait Parse {
    type Output;
    fn parse(&self, input: Input) -> ParseResult<Self::Output>;
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
    fn parse(&self, input: Input) -> ParseResult<()>;
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

    pub fn map_match<F, T, E>(self, f: F) -> Parser<MapMatch<M, F>>
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
