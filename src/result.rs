use std::fmt::Display;

use thiserror::Error;

pub type ParseResult<'a, O> = Result<(O, &'a str), RawEzpcError>;
pub type MatchResult<'a> = Result<&'a str, RawEzpcError>;

pub enum RawEzpcError {
    Mismatch {
        pos: *const u8,
    },
    Fatal {
        message: &'static str,
        pos: *const u8,
    },
    Recursion {
        max_depth: usize,
        parser_name: &'static str,
        pos: *const u8,
    },
}

/// Differs from RawEzpcError in that it has a Display implementation and that
/// the raw position pointers are converted in a printable Position struct.
/// This struct is aware of the input and can point to the exact location.
#[derive(Debug, Error)]
pub enum EzpcError<'a> {
    #[error("Parsing stopped before reaching end of input:\n{pos}")]
    PartialParse { pos: Position<'a> },
    #[error("{expected}\n{pos}")]
    Fatal {
        expected: &'static str,
        pos: Position<'a>,
    },
    #[error("Exceeded maximum recursion depth {max_depth} of parser {parser_name}:\n{pos}")]
    Recursion {
        max_depth: usize,
        parser_name: &'static str,
        pos: Position<'a>,
    },
}

impl<'a> EzpcError<'a> {
    pub fn from_raw(raw: RawEzpcError, source: &'a str) -> Self {
        match raw {
            RawEzpcError::Mismatch { pos } => EzpcError::PartialParse {
                pos: Position::from_ptr(source, pos),
            },
            RawEzpcError::Fatal {
                message: expected,
                pos,
            } => EzpcError::Fatal {
                expected,
                pos: Position::from_ptr(source, pos),
            },
            RawEzpcError::Recursion {
                max_depth,
                parser_name,
                pos,
            } => EzpcError::Recursion {
                max_depth,
                parser_name,
                pos: Position::from_ptr(source, pos),
            },
        }
    }
}

#[derive(Debug)]
pub struct Position<'a> {
    line: usize,
    column: usize,
    line_str: &'a str,
}

impl<'a> Position<'a> {
    pub fn from_ptr(source: &'a str, pos_ptr: *const u8) -> Self {
        let source_ptr = source.as_ptr() as usize;
        let pos_ptr = pos_ptr as usize;
        assert!(pos_ptr >= source_ptr);
        let slice_len = pos_ptr - source_ptr;

        let line = source[..slice_len].chars().filter(|&c| c == '\n').count() + 1;
        let line_start = source[..slice_len].rfind('\n').map_or(0, |pos| pos + 1);
        let column = source[line_start..slice_len].chars().count() + 1;

        let line_len = source[line_start..]
            .find('\r')
            .or(source[line_start..].find('\n'))
            .unwrap_or(source[line_start..].len());
        let line_str = &source[line_start..line_start + line_len];

        Self {
            line,
            column,
            line_str,
        }
    }
}

impl<'a> Display for Position<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pad = " ".repeat(self.line.ilog10() as usize + 1);
        writeln!(f, " --> line {}, column {}", self.line, self.column)?;
        writeln!(f, "{pad} |")?;
        writeln!(f, "{} | {}", self.line, self.line_str)?;
        write!(f, "{pad} | {}^", " ".repeat(self.column - 1))
    }
}
