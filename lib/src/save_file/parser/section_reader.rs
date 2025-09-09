use std::{collections::HashMap, fmt::Debug, io::Read};

use derive_more::{Display, Error, From};
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
#[derive(Debug, From, Display, Error)]
pub enum SectionReaderError<'err> {
    /// An unexpected token was encountered.
    #[display("reader encountered an unexpected token {:?} at {}: {}", _1, _0, _2)]
    UnexpectedToken(usize, Token<'err>, &'static str),
    /// An unknown token was encountered.
    UnknownToken(#[error(not(source))] u16),
    TextReaderError(TextReaderError),
    BinaryReaderError(BinaryReaderError),
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
                        if past_eq {
                            if let Some(key) = potential_key.take() {
                                self.attributes.insert(key, scalar.to_string());
                            }
                            past_eq = false;
                        } else {
                            potential_key = Some(scalar.to_string());
                        }
                    }
                    TextToken::Quoted(scalar) => {
                        if past_eq {
                            if let Some(key) = potential_key.take() {
                                self.attributes.insert(key, scalar.to_string());
                            }
                            past_eq = false;
                        }
                    }
                },
            }
        }
        return None;
    }

    /// Returns the value of an attribute with the given name.
    /// Attributes are key-value pairs in save file root.
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes.get(name).map(|s| s.as_str())
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
        let mut reader = TextSectionReader::new(TokenReader::from_slice(b""));
        assert!(reader.next().is_none());
    }

    #[test]
    fn test_single_section() {
        let mut reader = TextSectionReader::new(TokenReader::from_slice(b"test={a=1}"));
        assert_eq!(reader.next().unwrap().unwrap().get_name(), "test");
    }

    #[test]
    fn test_single_section_messy() {
        let mut reader =
            TextSectionReader::new(TokenReader::from_slice(b" \t\r   test={a=1}   \t\r "));
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        section.skip().unwrap();
        assert!(reader.next().is_none());
    }

    #[test]
    fn test_multiple_sections() {
        let mut reader =
            TextSectionReader::new(TokenReader::from_slice(b"test={a=1}test2={b=2}test3={c=3}"));
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
        let mut reader = TextSectionReader::new(TokenReader::from_slice(b"test={\x80=1}"));
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
    }

    #[test]
    fn test_mixed() {
        let mut reader = TextSectionReader::new(TokenReader::from_slice(
            b"a\na=b\ntest={a=1}test2={b=2}test3={c=3}",
        ));
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

    #[test]
    fn test_attributes() {
        let mut reader = TextSectionReader::new(TokenReader::from_slice(
            b"test=\"test\" \n test2=test \n section={test=test}",
        ));
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "section");
        assert_eq!(reader.attribute("test"), Some("test"));
        assert_eq!(reader.attribute("test2"), Some("test"));
    }

    #[test]
    fn test_broken_attribute() {
        let mut reader = TextSectionReader::new(TokenReader::from_slice(
            b"test=test test test \n test2=test \n section={test=test}",
        ));
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "section");
        assert_eq!(reader.attribute("test"), Some("test"));
        assert_eq!(reader.attribute("test2"), Some("test"));
    }
}
