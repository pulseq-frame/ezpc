mod parser;
mod range;
mod result;

pub use parser::{
    combinators::list,
    matchers::{is_a, none_of, one_of, tag},
    wrap::{DynMatch, DynParse, WrapMatcher, WrapParser},
    Match, Matcher, Parse, Parser,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn integer() -> Matcher<impl Match> {
        tag("0") | (one_of("123456789") + one_of("0123456789").repeat(0..))
    }

    fn number() -> Parser<impl Parse<Output = f64>> {
        let frac = tag(".") + one_of("0123456789").repeat(1..);
        let exp = one_of("eE") + one_of("+-").opt() + one_of("0123456789").repeat(1..);
        let number = tag("-").opt() + integer() + frac.opt() + exp.opt();

        number.try_map(f64::from_str)
    }

    #[test]
    fn parse_numbers() {
        assert!(number().parse("0.972") == Ok(0.972));
        assert!(number().parse("-12") == Ok(-12.0));
        assert!(number().parse("8.12e-3") == Ok(8.12e-3));
        assert!(number().parse("1E2") == Ok(1E2));
    }
}
