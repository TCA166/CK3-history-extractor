use std::collections::HashMap;

use lazy_static::lazy_static;

use jomini::binary::TokenResolver;

#[cfg(feature = "internal")]
const TOKENS: &'static str = include_str!("../../tokens_1.tok");

lazy_static! {
    pub static ref TOKENS_RESOLVER: TokenTranslator<'static> = TokenTranslator::default();
}

pub struct TokenTranslator<'a> {
    tokens: HashMap<u16, &'a str>,
}

impl<'a> Default for TokenTranslator<'a> {
    fn default() -> Self {
        #[cfg(feature = "internal")]
        {
            Self {
                tokens: TOKENS
                    .lines()
                    .map(|line| {
                        let mut parts = line.splitn(2, ' ');
                        let token = parts.next().unwrap();
                        let value = parts.next().unwrap().parse().unwrap();
                        (value, token)
                    })
                    .collect(),
            }
        }
        #[cfg(not(feature = "internal"))]
        {
            Self {
                tokens: HashMap::default(),
            }
        }
    }
}

impl TokenResolver for TokenTranslator<'_> {
    fn resolve(&self, token: u16) -> Option<&str> {
        self.tokens.get(&token).map(|v| &**v)
    }

    fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}
