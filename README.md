# Eazy Parser Combinator

This [parser combinator](https://en.wikipedia.org/wiki/Parser_combinator) is heavily inspired by [pom](https://crates.io/crates/pom), expect that:
- it uses static dispatch (json test parser is ~10x faster)
- it is utf-8 only, not meant for parsing binary inputs
- no lifetimes annotations are needed (which means that no-copy parsers are not possible)
- differentiates parers and matchers: no need to clutter code with ignored parser results
- better error support: Normal and fatal errors, which are reported with exact position

Coming Soon (TM): Examples, benchmarks, Getting Started etc...
