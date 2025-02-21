mod map;
pub use map::{GameMap, MapGenerator};

mod localizer;
use localizer::Localizer;
pub use localizer::{Localizable, LocalizationError, Localize};

mod loader;
pub use loader::GameDataLoader;
use serde::Serialize;

use super::parser::GameString;

#[derive(Serialize)]
pub struct GameData {
    map: Option<GameMap>,
    localizer: Localizer,
}

impl Localize<GameString> for GameData {
    fn lookup<K: AsRef<str>>(&self, key: K) -> Option<GameString> {
        self.localizer.lookup(key)
    }

    fn is_empty(&self) -> bool {
        self.localizer.is_empty()
    }
}

impl GameData {
    pub fn get_map(&self) -> Option<&GameMap> {
        self.map.as_ref()
    }

    pub fn get_localizer(&self) -> &Localizer {
        &self.localizer
    }
}
