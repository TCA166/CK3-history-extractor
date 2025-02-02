mod map;
pub use map::{GameMap, MapGenerator};

mod localizer;
use localizer::Localizer;
pub use localizer::{Localizable, LocalizationError, LocalizationStack, Localize};

mod loader;
pub use loader::GameDataLoader;

use super::parser::GameString;

pub struct GameData {
    map: Option<GameMap>,
    localizer: Localizer,
}

impl Localize for GameData {
    fn localize_query<K: AsRef<str>, S: AsRef<str>, F: Fn(&LocalizationStack) -> Option<S>>(
        &self,
        key: K,
        query: F,
    ) -> Result<GameString, LocalizationError> {
        self.localizer.localize_query(key, query)
    }
}

impl GameData {
    pub fn get_map(&self) -> Option<&GameMap> {
        self.map.as_ref()
    }
}
