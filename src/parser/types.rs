use std::fmt::{self, Debug};

use jomini::{
    binary::{Token as BinaryToken, TokenReader as BinaryTokenReader},
    text::{Token as TextToken, TokenReader as TextTokenReader},
};

/// An abstraction over the two [jomini] tape types: [jomini::TextTape] and [jomini::BinaryTape]
pub enum Tape<'a> {
    Text(TextTokenReader<&'a [u8]>),
    Binary(BinaryTokenReader<&'a [u8]>),
}

/// An abstraction over [jomini] tokens: [jomini::TextToken] and [jomini::BinaryToken]
pub enum Token<'a> {
    Text(TextToken<'a>),
    Binary(BinaryToken<'a>),
}

impl<'a> From<TextToken<'a>> for Token<'a> {
    fn from(token: TextToken<'a>) -> Self {
        Self::Text(token)
    }
}

impl<'a> From<BinaryToken<'a>> for Token<'a> {
    fn from(token: BinaryToken<'a>) -> Self {
        Self::Binary(token)
    }
}

impl Debug for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(token) => token.fmt(f),
            Self::Binary(token) => token.fmt(f),
        }
    }
}
