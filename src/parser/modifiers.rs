use super::{Match, Parse};
use crate::result::{MatchResult, ParseResult, RawEzpcError};

// These structs are returned by Parser / Matcher methods, so the fields must
// be public to the parent mod.

pub struct Fatal<T> {
    pub(super) parser_or_matcher: T,
    pub(super) expected: &'static str,
}

pub struct Reject<M> {
    pub(super) matcher: M,
    pub(super) expected: &'static str,
}

pub struct Repeat<T> {
    pub(super) parser_or_matcher: T,
    pub(super) start: usize,
    pub(super) end: usize,
}

pub struct Opt<T>(pub(super) T);

pub struct ValMatch<M, T> {
    pub(super) matcher: M,
    pub(super) value: T,
}

pub struct ValParse<P, T> {
    pub(super) parser: P,
    pub(super) value: T,
}

pub struct MapMatch<M, F> {
    pub(super) matcher: M,
    pub(super) map_func: F,
}

pub struct MapParse<P, F> {
    pub(super) parser: P,
    pub(super) map_func: F,
}

pub struct ConvertMatch<M, F> {
    pub(super) matcher: M,
    pub(super) map_func: F,
    pub(super) error_msg: &'static str,
}

pub struct ConvertParse<P, F> {
    pub(super) parser: P,
    pub(super) map_func: F,
    pub(super) error_msg: &'static str,
}

// Implementations for modified Parsers

impl<P: Parse> Parse for Fatal<P> {
    type Output = P::Output;

    fn apply<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.parser_or_matcher
            .apply(input)
            .map_err(|err| match err {
                RawEzpcError::Mismatch { pos } => RawEzpcError::Fatal {
                    message: self.expected,
                    pos,
                },
                _ => err,
            })
    }
}

impl<T: Parse> Parse for Repeat<T> {
    type Output = Vec<T::Output>;

    fn apply<'a>(&self, mut input: &'a str) -> ParseResult<'a, Self::Output> {
        let mut items = Vec::new();

        for _ in 0..=self.end {
            match self.parser_or_matcher.apply(input) {
                Ok((out, rest)) => {
                    items.push(out);
                    input = rest;
                }
                Err(err) => match err {
                    RawEzpcError::Mismatch { .. } => break,
                    _ => return Err(err),
                },
            }
        }

        if items.len() < self.start {
            Err(RawEzpcError::Mismatch {
                pos: input.as_ptr(),
            })
        } else {
            Ok((items, input))
        }
    }
}

impl<T: Parse> Parse for Opt<T> {
    type Output = Option<T::Output>;

    fn apply<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        match self.0.apply(input) {
            Ok((out, rest)) => Ok((Some(out), rest)),
            Err(err) => match err {
                RawEzpcError::Mismatch { .. } => Ok((None, input)),
                _ => return Err(err),
            },
        }
    }
}

// Implementations for modified Matchers

impl<M: Match> Match for Fatal<M> {
    fn apply<'a>(&self, input: &'a str) -> MatchResult<'a> {
        self.parser_or_matcher
            .apply(input)
            .map_err(|err| match err {
                RawEzpcError::Mismatch { pos } => RawEzpcError::Fatal {
                    message: self.expected,
                    pos,
                },
                _ => err,
            })
    }
}

impl<M: Match> Match for Reject<M> {
    fn apply<'a>(&self, input: &'a str) -> MatchResult<'a> {
        match self.matcher.apply(input) {
            Ok(_) =>  Err(RawEzpcError::Fatal { message: self.expected, pos: input.as_ptr() }),
            Err(err) => match err {
                RawEzpcError::Mismatch { .. } => Ok(input),
                _ => Err(err)
            }
        }
    }
}

impl<T: Match> Match for Repeat<T> {
    fn apply<'a>(&self, mut input: &'a str) -> MatchResult<'a> {
        let mut item_count = 0;

        for _ in 0..=self.end {
            match self.parser_or_matcher.apply(input) {
                Ok(rest) => {
                    item_count += 1;
                    input = rest;
                }
                Err(err) => match err {
                    RawEzpcError::Mismatch { .. } => break,
                    _ => return Err(err),
                },
            }
        }

        if item_count < self.start {
            Err(RawEzpcError::Mismatch {
                pos: input.as_ptr(),
            })
        } else {
            Ok(input)
        }
    }
}

impl<T: Match> Match for Opt<T> {
    fn apply<'a>(&self, input: &'a str) -> MatchResult<'a> {
        match self.0.apply(input) {
            Ok(rest) => Ok(rest),
            Err(err) => match err {
                RawEzpcError::Mismatch { .. } => Ok(input),
                _ => return Err(err),
            },
        }
    }
}

// Implementations for Mappers and Converters

impl<M: Match, T: Clone> Parse for ValMatch<M, T> {
    type Output = T;

    fn apply<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.matcher
            .apply(input)
            .map(|rest| (self.value.clone(), rest))
    }
}

impl<P: Parse, T: Clone> Parse for ValParse<P, T> {
    type Output = T;

    fn apply<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.parser
            .apply(input)
            .map(|(_, rest)| (self.value.clone(), rest))
    }
}

impl<M, F, O> Parse for MapMatch<M, F>
where
    M: Match,
    F: Fn(&str) -> O,
{
    type Output = O;

    fn apply<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.matcher
            .apply(input)
            .map(|rest| ((self.map_func)(consumed(input, rest)), rest))
    }
}

impl<P, F, O> Parse for MapParse<P, F>
where
    P: Parse,
    F: Fn(P::Output) -> O + 'static,
{
    type Output = O;

    fn apply<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.parser
            .apply(input)
            .map(|(tmp, rest)| ((self.map_func)(tmp), rest))
    }
}

impl<M, F, O, E> Parse for ConvertMatch<M, F>
where
    M: Match,
    F: Fn(&str) -> Result<O, E> + 'static,
    E: std::error::Error + 'static,
{
    type Output = O;

    fn apply<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.matcher
            .apply(input)
            .and_then(|rest| match (self.map_func)(consumed(input, rest)) {
                Ok(out) => Ok((out, rest)),
                Err(_) => Err(RawEzpcError::Fatal {
                    message: self.error_msg,
                    pos: rest.as_ptr(),
                }),
            })
    }
}

impl<P, F, O, E> Parse for ConvertParse<P, F>
where
    P: Parse,
    F: Fn(P::Output) -> Result<O, E> + 'static,
    E: std::error::Error + 'static,
{
    type Output = O;

    fn apply<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.parser
            .apply(input)
            .and_then(|(tmp, rest)| match (self.map_func)(tmp) {
                Ok(out) => Ok((out, rest)),
                Err(_) => Err(RawEzpcError::Fatal {
                    message: self.error_msg,
                    pos: rest.as_ptr(),
                }),
            })
    }
}

/// Helper function that returns the parsed part of the source str
fn consumed<'a>(source: &'a str, substr: &'a str) -> &'a str {
    let start_source = source.as_ptr() as usize;
    let start_substr = substr.as_ptr() as usize;
    assert!(start_substr > start_source);

    let advanced_by = start_substr - start_source;
    &source[..advanced_by]
}
