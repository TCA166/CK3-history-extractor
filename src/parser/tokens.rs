use jomini::binary::TokenResolver;

use phf::Map;

// This is generated at build time by the build.rs script.
#[cfg(feature = "tokens")]
include!(concat!(env!("OUT_DIR"), "/token_data.rs"));

/// A struct that translates tokens to strings
/// Essentially a wrapper around a static map that is generated at build time.
pub struct TokenTranslator {
    tokens: Option<&'static Map<u16, &'static str>>,
}

impl TokenResolver for TokenTranslator {
    fn resolve(&self, token: u16) -> Option<&str> {
        self.tokens.as_ref().unwrap().get(&token).map(|v| &**v)
    }

    fn is_empty(&self) -> bool {
        self.tokens.as_ref().unwrap().is_empty()
    }
}

/// A static instance of the token translator.
/// This is used to resolve tokens in the game data on runtime.
/// If the `tokens` feature is not enabled, this will be a no-op.
pub const TOKEN_TRANSLATOR: TokenTranslator = {
    #[cfg(feature = "tokens")]
    {
        TokenTranslator {
            tokens: Some(&TOKENS),
        }
    }
    #[cfg(not(feature = "tokens"))]
    {
        TokenTranslator { tokens: None }
    }
};
