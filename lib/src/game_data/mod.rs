mod map;
pub use map::{GameMap, MapGenerator, MapImage};

mod localizer;
use localizer::Localizer;
pub use localizer::{LocalizationError, Localize};

mod loader;
pub use loader::GameDataLoader;

use super::types::{GameId, GameString, HashMap};

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GameData {
    map: Option<GameMap>,
    localizer: Localizer,
    title_province_map: HashMap<GameId, GameString>,
}

impl Localize<GameString> for GameData {
    fn lookup<K: AsRef<str>>(&self, key: K) -> Option<GameString> {
        self.localizer.lookup(key)
    }

    fn is_empty(&self) -> bool {
        self.localizer.is_empty()
    }
}

/// A trait that allows an object to be localized.
pub trait Localizable {
    /// Localizes the object.
    fn localize(&mut self, localization: &GameData) -> Result<(), LocalizationError>;
    // I really don't like how this takes in GameData, but alas

    //TODO we should probably have a method here for simple queries, to allow some level of dynamic dispatch during localisation
}

impl GameData {
    pub fn get_map(&self) -> Option<&GameMap> {
        self.map.as_ref()
    }

    pub fn get_localizer(&self) -> &Localizer {
        &self.localizer
    }

    pub fn lookup_title(&self, id: &GameId) -> Option<GameString> {
        self.title_province_map.get(&id).cloned()
    }
}
