use std::fmt::Display;

use super::{Match, Matcher, Parse, Parser};
use crate::result::{MatcherError, ParseError, ParseResult};

pub struct List<P, M>
where
    P: Parse,
    M: Match,
{
    element: P,
    separator: M,
}

pub fn list<P, M>(element: Parser<P>, separator: Matcher<M>) -> Parser<List<P, M>>
where
    P: Parse,
    M: Match,
{
    Parser(List {
        element: element.0,
        separator: separator.0,
    })
}

impl<P, M> Parse for List<P, M>
where
    P: Parse,
    M: Match,
{
    type Output = Vec<P::Output>;

    fn apply<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        let mut items = Vec::new();

        match self.element.apply(input) {
            Ok((item, mut input)) => {
                items.push(item);
                // TODO: the separator might return a fatal error which we should handle?
                // But this would be strange since the separator should be simple (no recursion error)
                // and not expected, as it is optional (list has arbitrary length)
                while let Ok(rest) = self.separator.apply(input) {
                    match self.element.apply(rest) {
                        Ok((item, rest)) => {
                            items.push(item);
                            input = rest;
                        }
                        Err(err) => match err {
                            ParseError::Mismatch(_) => break,
                            _ => return Err(err),
                        },
                    }
                }
                Ok((items, input))
            }
            Err(err) => match err {
                ParseError::Mismatch(_) => Err(ParseError::Mismatch(MatcherError::List)),
                _ => Err(err),
            },
        }
    }
}

impl<P, M> Display for List<P, M>
where
    P: Parse,
    M: Match,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "List({}, {}", self.element, self.separator)
    }
}
