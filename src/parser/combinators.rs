use super::{Match, Matcher, Parse, Parser};
use crate::result::{ParseResult, RawEzpcError};

pub struct List<P, M>
where
    P: Parse,
    M: Match,
{
    element: P,
    separator: M,
    item_mismatch_error_msg: &'static str,
}

pub fn list<P, M>(
    element: Parser<P>,
    separator: Matcher<M>,
    item_mismatch_error_msg: &'static str,
) -> Parser<List<P, M>>
where
    P: Parse,
    M: Match,
{
    Parser(List {
        element: element.0,
        separator: separator.0,
        item_mismatch_error_msg,
    })
}

impl<P, M> Parse for List<P, M>
where
    P: Parse,
    M: Match,
{
    type Output = Vec<P::Output>;

    fn apply<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        // A list contains at least one element - if this errors the error is returned.
        // If it has success, we try to repeately parse separator + element
        self.element.apply(input).and_then(|(item, mut input)| {
            let mut items = vec![item];
            loop {
                // Parse as many list elements as possible
                match self.separator.apply(input) {
                    // Separator did not apply, list is finished, return
                    Err(RawEzpcError::Mismatch { .. }) => return Ok((items, input)),
                    // Separator returned some other error, forward it
                    Err(err) => return Err(err),
                    // Separator applied, now we expect a list element
                    Ok(rest) => match self.element.apply(rest) {
                        Ok((item, rest)) => {
                            items.push(item);
                            input = rest;
                        }
                        // List element didn't apply even though we had a separator -> fatal
                        Err(err) => {
                            return Err(match err {
                                RawEzpcError::Mismatch { pos } => RawEzpcError::Fatal {
                                    message: self.item_mismatch_error_msg,
                                    pos,
                                },
                                _ => err,
                            })
                        }
                    },
                }
            }
        })
    }
}
