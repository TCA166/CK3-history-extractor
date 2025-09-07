use std::{collections::HashMap, error, fmt::Debug, io::Read};

use derive_more::{Display, From};
use jomini::{
    binary::{
        ReaderError as BinaryReaderError, Token as BinaryToken, TokenReader as BinaryTokenReader,
        TokenResolver,
    },
    text::{
        Operator, ReaderError as TextReaderError, Token as TextToken,
        TokenReader as TextTokenReader,
    },
};

use super::{
    section::{BinarySection, TextSection},
    types::Token,
};

/// An error that occurred while reading sections from a tape.
#[derive(Debug, From, Display)]
pub enum SectionReaderError<'err> {
    /// An unexpected token was encountered.
    #[display("reader encountered an unexpected token {:?} at {}: {}", _1, _0, _2)]
    UnexpectedToken(usize, Token<'err>, &'static str),
    /// An unknown token was encountered.
    UnknownToken(u16),
    TextReaderError(TextReaderError),
    BinaryReaderError(BinaryReaderError),
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

pub struct TextSectionReader<R: Read> {
    tape: TextTokenReader<R>,
    attributes: HashMap<String, String>,
}

impl<R: Read> TextSectionReader<R> {
    pub fn new(tape: TextTokenReader<R>) -> Self {
        Self {
            tape,
            attributes: HashMap::new(),
        }
    }

    pub fn next(&mut self) -> Option<Result<TextSection<'_, R>, SectionReaderError<'_>>> {
        let mut potential_key = None;
        let mut past_eq = false;

        while let Some(res) = self.tape.next().transpose() {
            match res {
                Err(e) => {
                    return Some(Err(e.into()));
                }
                Ok(tok) => match tok {
                    TextToken::Open => {
                        if past_eq {
                            if let Some(key) = potential_key {
                                return Some(Ok(TextSection::new(&mut self.tape, key)));
                            }
                        }
                    }
                    TextToken::Close => {
                        return Some(Err(SectionReaderError::UnexpectedToken(
                            self.tape.position(),
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
        return None;
    }
}

pub struct BinarySectionReader<'resolver, R: Read> {
    tape: BinaryTokenReader<R>,
    token_resolver: &'resolver dyn TokenResolver,
    attributes: HashMap<String, String>,
}

impl<'resolver, R: Read> BinarySectionReader<'resolver, R> {
    pub fn new(tape: BinaryTokenReader<R>, token_resolver: &'resolver dyn TokenResolver) -> Self {
        Self {
            tape,
            token_resolver,
            attributes: HashMap::new(),
        }
    }

    pub fn next(
        &mut self,
    ) -> Option<Result<BinarySection<'_, 'resolver, R>, SectionReaderError<'_>>> {
        let mut potential_key = None;
        let mut past_eq = false;

        while let Some(res) = self.tape.next().transpose() {
            match res {
                Err(e) => {
                    return Some(Err(e.into()));
                }
                Ok(tok) => match tok {
                    BinaryToken::Open => {
                        if past_eq {
                            if let Some(key) = potential_key {
                                return Some(Ok(BinarySection::new(
                                    &mut self.tape,
                                    key,
                                    self.token_resolver,
                                )));
                            }
                        }
                    }
                    BinaryToken::Close => {
                        return Some(Err(SectionReaderError::UnexpectedToken(
                            self.tape.position(),
                            BinaryToken::Close.into(),
                            "unexpected close token",
                        )))
                    }
                    BinaryToken::Unquoted(token) => potential_key = Some(token.to_string()),
                    BinaryToken::Id(token) => match self.token_resolver.resolve(token) {
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
        return None;
    }
}

#[cfg(test)]
mod tests {
    use jomini::text::TokenReader;

    use super::{super::section::SaveFileSection, *};

    #[test]
    fn test_empty() {
        let mut reader = TextSectionReader {
            tape: TokenReader::from_slice(b""),
            attributes: HashMap::default(),
        };
        assert!(reader.next().is_none());
    }

    #[test]
    fn test_single_section() {
        let mut reader = TextSectionReader {
            tape: TokenReader::from_slice(b"test={a=1}"),
            attributes: HashMap::default(),
        };
        assert_eq!(reader.next().unwrap().unwrap().get_name(), "test");
    }

    #[test]
    fn test_single_section_messy() {
        let mut reader = TextSectionReader {
            tape: TokenReader::from_slice(b" \t\r   test={a=1}   \t\r "),
            attributes: HashMap::default(),
        };
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        section.skip().unwrap();
        assert!(reader.next().is_none());
    }

    #[test]
    fn test_multiple_sections() {
        let mut reader = TextSectionReader {
            tape: TokenReader::from_slice(b"test={a=1}test2={b=2}test3={c=3}"),
            attributes: HashMap::default(),
        };
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        section.skip().unwrap();
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test2");
        section.skip().unwrap();
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test3");
        section.skip().unwrap();
        assert!(reader.next().is_none());
    }

    #[test]
    fn test_non_ascii_key() {
        let mut reader = TextSectionReader {
            tape: TokenReader::from_slice(b"test={\x80=1}"),
            attributes: HashMap::default(),
        };
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
    }

    #[test]
    fn test_mixed() {
        let mut reader = TextSectionReader {
            tape: TokenReader::from_slice(b"a\na=b\ntest={a=1}test2={b=2}test3={c=3}"),
            attributes: HashMap::default(),
        };
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        section.skip().unwrap();
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test2");
        section.skip().unwrap();
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test3");
        section.skip().unwrap();
    }
}
