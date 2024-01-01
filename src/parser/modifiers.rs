use std::fmt::Display;

use super::{Match, Parse};
use crate::result::{MatchResult, ParseResult, RawEzpcError};

pub struct Fatal<T> {
    pub parser_or_matcher: T,
    pub expected: &'static str,
}

pub struct CheckMatch<M> {
    pub matcher: M,
    pub expected: &'static str,
}

pub struct CheckParse<P> {
    pub parser: P,
    pub expected: &'static str,
}

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

pub struct ValParse<P, T> {
    pub(crate) parser: P,
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

pub struct TryMapMatch<M, F> {
    pub(crate) matcher: M,
    pub(crate) map_func: F,
}

pub struct TryMapParse<P, F> {
    pub(crate) parser: P,
    pub(crate) map_func: F,
}

// Display Implementations

impl<T: Display> Display for Fatal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "!{}: {}", self.parser_or_matcher, self.expected)
    }
}

impl<M: Match> Display for CheckMatch<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "!{}: {}", self.matcher, self.expected)
    }
}

impl<P: Parse> Display for CheckParse<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "!{}: {}", self.parser, self.expected)
    }
}

impl<T: Display> Display for Repeat<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.end == usize::MAX {
            write!(f, "{}*({}..)", self.parser_or_matcher, self.start)
        } else if self.end + 1 == self.start {
            write!(f, "{}*({})", self.parser_or_matcher, self.start)
        } else {
            write!(
                f,
                "{}*({}..={})",
                self.parser_or_matcher, self.start, self.end
            )
        }
    }
}

impl<T: Display> Display for Opt<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}?", self.0)
    }
}

impl<M: Display, T> Display for ValMatch<M, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.val(...)", self.matcher)
    }
}

impl<P: Display, T> Display for ValParse<P, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.val(...)", self.parser)
    }
}

impl<M: Display, F> Display for MapMatch<M, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.map(...)", self.matcher)
    }
}

impl<P: Display, F> Display for MapParse<P, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.map(...)", self.parser)
    }
}

impl<M: Display, F> Display for TryMapMatch<M, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.try_map(...)", self.matcher)
    }
}

impl<P: Display, F> Display for TryMapParse<P, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.try_map(...)", self.parser)
    }
}

// Implementations for modified Parsers

impl<P: Parse> Parse for Fatal<P> {
    type Output = P::Output;

    fn apply<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.parser_or_matcher
            .apply(input)
            .map_err(|err| match err {
                RawEzpcError::PartialParse { pos } => RawEzpcError::Fatal {
                    expected: self.expected,
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
                    RawEzpcError::PartialParse { .. } => break,
                    _ => return Err(err),
                },
            }
        }

        if items.len() < self.start {
            Err(RawEzpcError::PartialParse {
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
                RawEzpcError::PartialParse { .. } => Ok((None, input)),
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
                RawEzpcError::PartialParse { pos } => RawEzpcError::Fatal {
                    expected: self.expected,
                    pos,
                },
                _ => err,
            })
    }
}

impl<M: Match> Match for CheckMatch<M> {
    fn apply<'a>(&self, input: &'a str) -> MatchResult<'a> {
        match self.matcher.apply(input) {
            Ok(_) => Ok(input), // We want to ensure it applies without applying it
            Err(err) => Err(match err {
                RawEzpcError::PartialParse { pos } => RawEzpcError::Fatal {
                    expected: self.expected,
                    pos,
                },
                _ => err,
            }),
        }
    }
}

impl<P: Parse> Match for CheckParse<P> {
    fn apply<'a>(&self, input: &'a str) -> MatchResult<'a> {
        match self.parser.apply(input) {
            Ok(_) => Ok(input), // We want to ensure it applies without applying it
            Err(err) => Err(match err {
                RawEzpcError::PartialParse { pos } => RawEzpcError::Fatal {
                    expected: self.expected,
                    pos,
                },
                _ => err,
            }),
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
                    RawEzpcError::PartialParse { .. } => break,
                    _ => return Err(err),
                },
            }
        }

        if item_count < self.start {
            Err(RawEzpcError::PartialParse {
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
                RawEzpcError::PartialParse { .. } => Ok(input),
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

impl<M, F, O, E> Parse for TryMapMatch<M, F>
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
                Err(_) => Err(RawEzpcError::PartialParse { pos: rest.as_ptr() }),
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

    fn apply<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        self.parser
            .apply(input)
            .and_then(|(tmp, rest)| match (self.map_func)(tmp) {
                Ok(out) => Ok((out, rest)),
                Err(_) => Err(RawEzpcError::PartialParse { pos: rest.as_ptr() }),
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
