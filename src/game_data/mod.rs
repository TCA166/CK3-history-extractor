mod map;
pub use map::{GameMap, MapGenerator};

mod localizer;
use localizer::Localizer;
pub use localizer::{Localizable, Localize};

mod loader;
pub use loader::GameDataLoader;

use super::parser::GameString;

pub struct GameData {
    map: Option<GameMap>,
    localizer: Localizer,
}

impl Localize for GameData {
    fn localize(&mut self, key: &str) -> GameString {
        self.localizer.localize(key)
    }
}

impl GameData {
    pub fn get_map(&self) -> Option<&GameMap> {
        self.map.as_ref()
    }
}
