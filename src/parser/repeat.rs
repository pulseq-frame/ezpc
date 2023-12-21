use crate::{
    input::Input,
    result::{ParseError, ParseResult},
};

use super::{Match, Parse};

pub struct Repeat<T> {
    pub(crate) parser_or_matcher: T,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl<T: Parse> Parse for Repeat<T> {
    type Output = Vec<T::Output>;

    fn parse(&self, mut input: Input) -> ParseResult<Self::Output> {
        let mut items = Vec::new();

        for _ in 0..=self.end {
            if let Ok((out, rest)) = self.parser_or_matcher.parse(input.clone()) {
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

impl<T: Match> Match for Repeat<T> {
    fn parse(&self, mut input: Input) -> ParseResult<()> {
        let mut item_count = 0;

        for _ in 0..=self.end {
            if let Ok(((), rest)) = self.parser_or_matcher.parse(input.clone()) {
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
            Ok(((), input))
        }
    }
}
