use std::fmt::{self, Debug};

use jomini::{BinaryTape, BinaryToken, TextTape, TextToken};

/// An abstraction over the two [jomini] tape types: [jomini::TextTape] and [jomini::BinaryTape]
pub enum Tape<'a> {
    Text(TextTape<'a>),
    Binary(BinaryTape<'a>),
}

impl<'a> Tape<'a> {
    /// Gets the abstraction over the tokens held by this tape
    pub fn tokens(&'a self) -> Tokens<'a, 'a> {
        match self {
            Self::Binary(bin) => Tokens::Binary(bin.tokens()),
            Self::Text(text) => Tokens::Text(text.tokens()),
        }
    }
}

/// An abstraction over the two token types: [jomini::TextToken] and [jomini::BinaryToken]
pub enum Tokens<'token, 'data> {
    Text(&'token [TextToken<'data>]),
    Binary(&'token [BinaryToken<'data>]),
}

impl<'token, 'data> Tokens<'token, 'data> {
    pub fn len(&self) -> usize {
        match self {
            Self::Binary(bin) => bin.len(),
            Self::Text(text) => text.len(),
        }
    }
}

/// An abstraction over [jomini] tokens: [jomini::TextToken] and [jomini::BinaryToken]
pub enum Token<'a> {
    Text(TextToken<'a>),
    Binary(BinaryToken<'a>),
}

impl<'a> Token<'a> {
    pub fn from_text(token: &TextToken<'a>) -> Self {
        Self::Text(token.to_owned())
    }

    pub fn from_binary(token: &BinaryToken<'a>) -> Self {
        Self::Binary(token.to_owned())
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
