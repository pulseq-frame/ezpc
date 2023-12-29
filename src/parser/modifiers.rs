use std::fmt::Display;

use crate::result::{MatchResult, MatcherError, ParseError, ParseResult};

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

impl<T: Display> Display for Repeat<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.end == usize::MAX {
            write!(f, "{}.repeat({}..)", self.parser_or_matcher, self.start)
        } else if self.end + 1 == self.start {
            write!(f, "{}.repeat({})", self.parser_or_matcher, self.start)
        } else {
            write!(
                f,
                "{}.repeat({}..={})",
                self.parser_or_matcher, self.start, self.end
            )
        }
    }
}

impl<T: Display> Display for Opt<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.opt()", self.0)
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

impl<T: Parse> Parse for Repeat<T> {
    type Output = Vec<T::Output>;

    fn apply<'a>(&self, mut input: &'a str, depth: usize) -> ParseResult<'a, Self::Output> {
        let mut items = Vec::new();

        for _ in 0..=self.end {
            match self.parser_or_matcher.apply(input, depth) {
                Ok((out, rest)) => {
                    items.push(out);
                    input = rest;
                }
                Err(err) => match err {
                    ParseError::RecursionDepth(_) => return Err(err),
                    _ => break,
                },
            }
        }

        if items.len() < self.start {
            Err(ParseError::Mismatch(MatcherError::Repeat {
                min: self.start,
                count: items.len(),
            }))
        } else {
            Ok((items, input))
        }
    }
}

impl<T: Parse> Parse for Opt<T> {
    type Output = Option<T::Output>;

    fn apply<'a>(&self, input: &'a str, depth: usize) -> ParseResult<'a, Self::Output> {
        match self.0.apply(input, depth) {
            Ok((out, rest)) => Ok((Some(out), rest)),
            Err(err) => match err {
                ParseError::RecursionDepth(_) => Err(err),
                _ => Ok((None, input)),
            },
        }
    }
}

// Implementations for modified Matchers

impl<T: Match> Match for Repeat<T> {
    fn apply<'a>(&self, mut input: &'a str, depth: usize) -> MatchResult<'a> {
        let mut item_count = 0;

        for _ in 0..=self.end {
            match self.parser_or_matcher.apply(input, depth) {
                Ok(rest) => {
                    item_count += 1;
                    input = rest;
                }
                Err(err) => match err {
                    ParseError::RecursionDepth(_) => return Err(err),
                    _ => break,
                },
            }
        }

        if item_count < self.start {
            Err(ParseError::Mismatch(MatcherError::Repeat {
                min: self.start,
                count: item_count,
            }))
        } else {
            Ok(input)
        }
    }
}

impl<T: Match> Match for Opt<T> {
    fn apply<'a>(&self, input: &'a str, depth: usize) -> MatchResult<'a> {
        match self.0.apply(input, depth) {
            Ok(rest) => Ok(rest),
            Err(err) => match err {
                ParseError::RecursionDepth(_) => Err(err),
                _ => Ok(input),
            },
        }
    }
}

// Implementations for Mappers and Converters

impl<M: Match, T: Clone> Parse for ValMatch<M, T> {
    type Output = T;

    fn apply<'a>(&self, input: &'a str, depth: usize) -> ParseResult<'a, Self::Output> {
        self.matcher
            .apply(input, depth)
            .map(|rest| (self.value.clone(), rest))
    }
}

impl<P: Parse, T: Clone> Parse for ValParse<P, T> {
    type Output = T;

    fn apply<'a>(&self, input: &'a str, depth: usize) -> ParseResult<'a, Self::Output> {
        self.parser
            .apply(input, depth)
            .map(|(_, rest)| (self.value.clone(), rest))
    }
}

impl<M, F, O> Parse for MapMatch<M, F>
where
    M: Match,
    F: Fn(&str) -> O,
{
    type Output = O;

    fn apply<'a>(&self, input: &'a str, depth: usize) -> ParseResult<'a, Self::Output> {
        self.matcher
            .apply(input, depth)
            .map(|rest| ((self.map_func)(consumed(input, rest)), rest))
    }
}

impl<P, F, O> Parse for MapParse<P, F>
where
    P: Parse,
    F: Fn(P::Output) -> O + 'static,
{
    type Output = O;

    fn apply<'a>(&self, input: &'a str, depth: usize) -> ParseResult<'a, Self::Output> {
        self.parser
            .apply(input, depth)
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

    fn apply<'a>(&self, input: &'a str, depth: usize) -> ParseResult<'a, Self::Output> {
        self.matcher.apply(input, depth).and_then(|rest| {
            match (self.map_func)(consumed(input, rest)) {
                Ok(out) => Ok((out, rest)),
                Err(err) => Err(ParseError::Mismatch(MatcherError::Boxed(err.into()))),
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

    fn apply<'a>(&self, input: &'a str, depth: usize) -> ParseResult<'a, Self::Output> {
        self.parser
            .apply(input, depth)
            .and_then(|(tmp, rest)| match (self.map_func)(tmp) {
                Ok(out) => Ok((out, rest)),
                Err(err) => Err(ParseError::Mismatch(MatcherError::Boxed(err.into()))),
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
