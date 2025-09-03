use std::path::Path;

use serde::Serialize;

use crate::types::WrapperMut;

use super::{
    super::{
        display::{Grapher, ProceduralPath, Renderable},
        game_data::{GameData, Localizable, LocalizationError, Localize, MapGenerator, MapImage},
        jinja_env::FAITH_TEMPLATE_NAME,
        parser::{GameObjectMap, GameObjectMapping, GameState, ParsingError},
        types::{GameString, Wrapper},
    },
    Character, EntityRef, Finalize, FromGameObject, GameObjectDerived, GameObjectEntity, GameRef,
    Title,
};

/// A struct representing a faith in the game
#[derive(Serialize)]
pub struct Faith {
    name: GameString,
    tenets: Vec<GameString>,
    head_title: Option<GameRef<Title>>,
    head: Option<GameRef<Character>>,
    fervor: f32,
    doctrines: Vec<GameString>,
}

impl FromGameObject for Faith {
    fn from_game_object(
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError> {
        let mut val = Self {
            name: base
                .get("name")
                .or(base.get("template"))
                .map(|v| v.as_string())
                .transpose()?
                .unwrap(),
            fervor: base.get_real("fervor")? as f32,
            head_title: base
                .get_game_id("religious_head")
                .map(|v| game_state.get_title(&v))
                .ok(),
            head: None,
            tenets: Vec::new(),
            doctrines: Vec::new(),
        };
        for t in base.get_object("doctrine")?.as_array()? {
            let s = t.as_string()?;
            if s.contains("tenet") {
                val.tenets.push(s);
            } else {
                val.doctrines.push(s);
            }
        }
        return Ok(val);
    }
}

impl Finalize for GameRef<Faith> {
    fn finalize(&mut self) {
        if let Some(faith) = self.get_internal_mut().inner_mut() {
            if let Some(head_title) = &faith.head_title {
                if let Some(head) = head_title.get_internal().inner() {
                    faith.head = head.get_holder()
                }
            }
        }
    }
}

impl GameObjectDerived for Faith {
    fn get_name(&self) -> GameString {
        self.name.clone()
    }

    fn get_references<E: From<EntityRef>, C: Extend<E>>(&self, collection: &mut C) {
        if let Some(head) = &self.head {
            collection.extend([E::from(head.clone().into())]);
        }
    }
}

impl ProceduralPath for Faith {
    fn get_subdir() -> &'static str {
        "faiths"
    }
}

impl Renderable for GameObjectEntity<Faith> {
    fn get_template() -> &'static str {
        FAITH_TEMPLATE_NAME
    }

    fn render(
        &self,
        path: &Path,
        game_state: &GameState,
        grapher: Option<&Grapher>,
        data: &GameData,
    ) {
        if let Some(grapher) = grapher {
            let mut buf = path.join(Faith::get_subdir());
            buf.push(self.id.to_string() + ".svg");
            grapher.create_faith_graph(self.id, &buf);
        }
        if let Some(map) = data.get_map() {
            let filter = |title: &Title| {
                if let Title::County { faith, .. } = title {
                    if let Some(c_faith) = faith {
                        return c_faith.get_internal().id == self.id;
                    }
                }
                return false;
            };
            let keys = game_state.get_baronies_of_counties(filter);
            if !keys.is_empty() {
                let mut buf = path.join(Faith::get_subdir());
                buf.push(self.id.to_string() + ".png");
                let mut faith_map = map.create_map_flat(keys, [70, 255, 70]);
                if let Some(inner) = self.inner() {
                    faith_map.draw_text(format!("Map of the {} faith", &inner.name));
                }
                faith_map.save_in_thread(&buf);
            }
        }
    }
}

impl Localizable for Faith {
    fn localize(&mut self, localization: &GameData) -> Result<(), LocalizationError> {
        self.name = localization.localize(&self.name)?;
        for tenet in self.tenets.iter_mut() {
            *tenet = localization.localize(tenet.to_string() + "_name")?;
        }
        for doctrine in self.doctrines.iter_mut() {
            *doctrine = localization.localize(doctrine.to_string() + "_name")?;
        }
        Ok(())
    }
}
