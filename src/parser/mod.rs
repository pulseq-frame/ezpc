pub mod combinators;
pub mod combine_ops;
pub mod matchers;
pub mod modifiers;
pub mod wrap;

use crate::{
    range::RangeArgument,
    result::{EzpcError, MatchResult, ParseResult, Position},
};
use modifiers::{
    ConvertMatch, ConvertParse, Fatal, MapMatch, MapParse, Opt, Reject, Repeat, ValMatch, ValParse,
};

pub trait Parse {
    type Output;
    fn apply<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output>;
}

pub struct Parser<T: Parse>(T);

impl<P: Parse> Parser<P> {
    pub fn parse_all(&self, source: &str) -> Result<P::Output, EzpcError> {
        match self.0.apply(source) {
            Ok((out, rest)) => {
                if rest.is_empty() {
                    Ok(out)
                } else {
                    Err(EzpcError::PartialParse {
                        pos: Position::from_ptr(source, rest.as_ptr()),
                    })
                }
            }
            Err(raw) => Err(EzpcError::from_raw(raw, source)),
        }
    }

    pub fn fatal(self, expected: &'static str) -> Parser<Fatal<P>> {
        Parser(Fatal {
            parser_or_matcher: self.0,
            expected,
        })
    }

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

    pub fn convert<F, O, E>(self, f: F, error_msg: &'static str) -> Parser<ConvertParse<P, F>>
    where
        F: Fn(P::Output) -> Result<O, E> + 'static,
        E: std::error::Error + 'static,
    {
        Parser(ConvertParse {
            parser: self.0,
            map_func: f,
            error_msg,
        })
    }
}

pub trait Match {
    fn apply<'a>(&self, input: &'a str) -> MatchResult<'a>;
}

pub struct Matcher<M: Match>(M);

impl<M: Match> Matcher<M> {
    pub fn match_all(&self, source: &str) -> Result<(), EzpcError> {
        match self.0.apply(source) {
            Ok(rest) => {
                if rest.is_empty() {
                    Ok(())
                } else {
                    Err(EzpcError::PartialParse {
                        pos: Position::from_ptr(source, rest.as_ptr()),
                    })
                }
            }
            Err(raw) => Err(EzpcError::from_raw(raw, source)),
        }
    }

    pub fn fatal(self, expected: &'static str) -> Matcher<Fatal<M>> {
        Matcher(Fatal {
            parser_or_matcher: self.0,
            expected,
        })
    }

    pub fn reject(self, expected: &'static str) -> Matcher<Reject<M>> {
        Matcher(Reject {
            matcher: self.0,
            expected,
        })
    }

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

    pub fn convert<F, T, E>(
        self,
        map_func: F,
        error_msg: &'static str,
    ) -> Parser<ConvertMatch<M, F>>
    where
        F: Fn(&str) -> Result<T, E> + 'static,
        E: std::error::Error + 'static,
    {
        Parser(ConvertMatch {
            matcher: self.0,
            map_func,
            error_msg,
        })
    }
}
