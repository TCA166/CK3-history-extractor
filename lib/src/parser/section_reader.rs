use std::{
    error,
    fmt::{self, Debug, Display},
};

use derive_more::From;
use jomini::{
    binary::{ReaderError as BinaryReaderError, Token as BinaryToken, TokenResolver},
    text::{Operator, ReaderError as TextReaderError, Token as TextToken},
};

use super::{
    section::Section,
    tokens::TOKEN_TRANSLATOR,
    types::{Tape, Token},
};

/// An error that occurred while reading sections from a tape.
#[derive(Debug, From)]
pub enum SectionReaderError<'err> {
    /// An unexpected token was encountered.
    UnexpectedToken(usize, Token<'err>, &'static str),
    /// An unknown token was encountered.
    UnknownToken(u16),
    TextReaderError(TextReaderError),
    BinaryReaderError(BinaryReaderError),
}

impl<'err> Display for SectionReaderError<'err> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedToken(pos, tok, desc) => {
                write!(
                    f,
                    "reader encountered an unexpected token {:?} at {}: {}",
                    tok, pos, desc
                )
            }
            Self::UnknownToken(token) => {
                write!(f, "reader encountered an unknown token {}", token)
            }
            Self::TextReaderError(e) => {
                write!(f, "text reader encountered an error: {}", e)
            }
            Self::BinaryReaderError(e) => {
                write!(f, "binary reader encountered an error: {}", e)
            }
        }
    }
}

impl<'err> error::Error for SectionReaderError<'err> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::TextReaderError(e) => Some(e),
            Self::BinaryReaderError(e) => Some(e),
            _ => None,
        }
    }
}

/// Essentially an iterator over sections in a tape.
/// Previously a struct, but this is simpler, and makes the borrow checker happy.
/// Returns None if there are no more sections. Otherwise, returns the next section, or reports an error.
pub fn yield_section<'tape, 'data: 'tape>(
    tape: &'tape mut Tape<'data>,
) -> Option<Result<Section<'tape, 'data>, SectionReaderError<'data>>> {
    let mut potential_key = None;
    let mut past_eq = false;
    match tape {
        Tape::Text(text) => {
            while let Some(res) = text.next().transpose() {
                match res {
                    Err(e) => {
                        return Some(Err(e.into()));
                    }
                    Ok(tok) => match tok {
                        TextToken::Open => {
                            if past_eq {
                                if let Some(key) = potential_key {
                                    return Some(Ok(Section::new(tape, key)));
                                }
                            }
                        }
                        TextToken::Close => {
                            return Some(Err(SectionReaderError::UnexpectedToken(
                                text.position(),
                                TextToken::Close.into(),
                                "unexpected close token",
                            )))
                        }
                        TextToken::Operator(op) => {
                            if op == Operator::Equal {
                                past_eq = true;
                            } else {
                                past_eq = false;
                            }
                        }
                        TextToken::Unquoted(scalar) => {
                            potential_key = Some(scalar.to_string());
                        }
                        _ => {
                            past_eq = false;
                            potential_key = None;
                        }
                    },
                }
            }
        }
        Tape::Binary(binary) => {
            while let Some(res) = binary.next().transpose() {
                match res {
                    Err(e) => {
                        return Some(Err(e.into()));
                    }
                    Ok(tok) => match tok {
                        BinaryToken::Open => {
                            if past_eq {
                                if let Some(key) = potential_key {
                                    return Some(Ok(Section::new(tape, key)));
                                }
                            }
                        }
                        BinaryToken::Close => {
                            return Some(Err(SectionReaderError::UnexpectedToken(
                                tape.position(),
                                BinaryToken::Close.into(),
                                "unexpected close token",
                            )))
                        }
                        BinaryToken::Unquoted(token) => potential_key = Some(token.to_string()),
                        BinaryToken::Id(token) => match TOKEN_TRANSLATOR.resolve(token) {
                            Some(key) => {
                                potential_key = Some(key.to_string());
                            }
                            None => {
                                return Some(Err(SectionReaderError::UnknownToken(token)));
                            }
                        },
                        BinaryToken::Equal => {
                            past_eq = true;
                        }
                        _ => {
                            past_eq = false;
                            potential_key = None;
                        }
                    },
                }
            }
        }
    }
    return None;
}

#[cfg(test)]
mod tests {
    use jomini::text::TokenReader;

    use super::*;

    #[test]
    fn test_empty() {
        let mut tape = Tape::Text(TokenReader::from_slice(b""));
        assert!(yield_section(&mut tape).is_none());
    }

    #[test]
    fn test_single_section() {
        let mut tape = Tape::Text(TokenReader::from_slice(b"test={a=1}"));
        let section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
    }

    #[test]
    fn test_single_section_messy() {
        let mut tape = Tape::Text(TokenReader::from_slice(b" \t\r   test={a=1}   \t\r "));
        let mut section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        section.skip().unwrap();
        assert!(yield_section(&mut tape).is_none());
    }

    #[test]
    fn test_multiple_sections() {
        let mut tape = Tape::Text(TokenReader::from_slice(b"test={a=1}test2={b=2}test3={c=3}"));
        let mut section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        section.skip().unwrap();
        let mut section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "test2");
        section.skip().unwrap();
        let mut section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "test3");
        section.skip().unwrap();
        assert!(yield_section(&mut tape).is_none());
    }

    #[test]
    fn test_non_ascii_key() {
        let mut tape = Tape::Text(TokenReader::from_slice(b"test={\x80=1}"));
        let section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
    }

    #[test]
    fn test_mixed() {
        let mut tape = Tape::Text(TokenReader::from_slice(
            b"a\na=b\ntest={a=1}test2={b=2}test3={c=3}",
        ));
        let mut section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        section.skip().unwrap();
        let mut section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "test2");
        section.skip().unwrap();
        let mut section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "test3");
        section.skip().unwrap();
    }
}
