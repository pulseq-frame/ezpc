mod parser;
mod range;
mod result;
#[cfg(test)]
mod tests;

pub use parser::{
    combinators::list,
    matchers::{eof, is_a, none_of, one_of, tag},
    wrap::{WrapMatcher, WrapParser},
    Match, Matcher, Parse, Parser,
};
