use std::{
    error,
    fmt::{self, Debug, Display},
};

use jomini::{
    binary::{ReaderError as BinaryReaderError, Token as BinaryToken},
    text::{Operator, ReaderError as TextReaderError, Token as TextToken},
};

use super::{
    section::Section,
    types::{Tape, Token},
};

/// An error that occurred while reading sections from a tape.
#[derive(Debug)]
pub enum SectionReaderError<'err> {
    /// An unexpected token was encountered.
    UnexpectedToken(usize, Token<'err>, &'static str),
    /// An unknown token was encountered.
    UnknownToken(u16),
    TextReaderError(TextReaderError),
    BinaryReaderError(BinaryReaderError),
}

impl From<TextReaderError> for SectionReaderError<'_> {
    fn from(e: TextReaderError) -> Self {
        Self::TextReaderError(e)
    }
}

impl From<BinaryReaderError> for SectionReaderError<'_> {
    fn from(e: BinaryReaderError) -> Self {
        Self::BinaryReaderError(e)
    }
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

/// Resolve a token to a section name.
fn token_resolver(token: &u16) -> Option<&'static str> {
    Some(match token {
        12629 => "meta_data",
        1365 => "variables",
        13665 => "traits_lookup",
        1719 => "provinces",
        10198 => "landed_titles",
        10805 => "dynasties",
        10133 => "character_lookup",
        13582 => "deleted_characters",
        11494 => "living",
        11496 => "dead_unprunable",
        1763 => "characters",
        10232 => "units",
        10268 => "triggered_event", // huge block of identical tokens
        10219 => "played_character",
        12748 => "currently_played_characters",
        10603 => "armies",
        14365 => "activity_manager",
        10382 => "opinions",
        10467 => "relations",
        10507 => "schemes",
        11164 => "stories",
        10605 => "combats",
        10412 => "pending_character_interactions",
        10497 => "secrets",
        12428 => "mercenary_company_manager",
        10672 => "vassal_contracts",
        10236 => "religion",
        11070 => "wars",
        11348 => "sieges",
        11623 => "succession",
        11650 => "holdings",
        11719 => "county_manager",
        11753 => "fleet_manager",
        11800 => "council_task_manager",
        11825 => "important_action_manager",
        12002 => "faction_manager",
        12178 => "culture_manager",
        12450 => "holy_orders",
        10975 => "ai",
        1688 => "game_rules",
        13214 => "raid",
        13386 => "ironman_manager",
        10137 => "coat_of_arms",
        13715 => "artifacts",
        13765 => "inspirations_manager",
        13964 => "court_positions",
        14164 => "struggle_manager",
        13828 => "character_memory_manager",
        14115 => "diarchies",
        14263 => "travel_plans",
        14020 => "accolades",
        14084 => "tax_slot_manager",
        14807 => "epidemics",
        14939 => "legends",
        _ => return None,
    })
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
                                binary.position(),
                                BinaryToken::Close.into(),
                                "unexpected close token",
                            )))
                        }
                        BinaryToken::Unquoted(token) => potential_key = Some(token.to_string()),
                        BinaryToken::Id(token) => match token_resolver(&token) {
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
        let section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        assert!(yield_section(&mut tape).is_none());
    }

    #[test]
    fn test_multiple_sections() {
        let mut tape = Tape::Text(TokenReader::from_slice(b"test={a=1}test2={b=2}test3={c=3}"));
        let section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        let section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "test2");
        let section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "test3");
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
        let section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        let section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "test2");
        let section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "test3");
    }
}
