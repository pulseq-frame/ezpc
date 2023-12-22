use super::{Match, Matcher, Parse, Parser};
use crate::result::{ParseError, ParseResult};

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

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        let mut items = Vec::new();

        if let Ok((item, mut input)) = self.element.parse(input) {
            items.push(item);
            while let Ok(((), rest)) = self.separator.parse(input) {
                match self.element.parse(rest) {
                    Ok((item, rest)) => {
                        items.push(item);
                        input = rest;
                    }
                    Err(_) => break,
                }
            }
            Ok((items, input))
        } else {
            Err(ParseError::Generic(
                "List should match at least one element".into(),
            ))
        }
    }
}
