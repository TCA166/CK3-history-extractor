use std::{cell::Ref, path::Path};

use serde::Serialize;

use super::{
    super::{
        display::{Grapher, ProceduralPath, Renderable},
        game_data::{GameData, Localizable, LocalizationError, Localize, MapGenerator},
        jinja_env::FAITH_TEMPLATE_NAME,
        parser::{GameId, GameObjectMap, GameObjectMapping, GameState, GameString, ParsingError},
        types::Wrapper,
    },
    Character, GameObjectDerived, GameObjectEntity, Shared, Title,
};

/// A struct representing a faith in the game
#[derive(Serialize, Debug)]
pub struct Faith {
    name: GameString,
    tenets: Vec<GameString>,
    head: Option<Shared<Character>>,
    fervor: f32,
    doctrines: Vec<GameString>,
}

impl GameObjectDerived for Faith {
    fn get_name(&self) -> GameString {
        self.name.clone()
    }

    fn get_references<T: GameObjectDerived, E: From<GameObjectEntity<T>>, C: Extend<E>>(
        &self,
        collection: &mut C,
    ) {
        if let Some(head) = &self.head {
            collection.extend([E::from(head.clone().into())]);
        }
    }

    fn new(
        id: GameId,
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError>
    where
        Self: Sized,
    {
        let mut val = Self {
            name: base
                .get("name")
                .or(base.get("template"))
                .map(|v| v.as_string()?),
            fervor: base.get_real("fervor"),
            head: base
                .get_game_id("religious_head")
                .map(|v| game_state.get_title(&v).get_internal().get_holder()),
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

impl ProceduralPath for Faith {
    fn get_subdir() -> &'static str {
        "faiths"
    }
}

impl Renderable for Faith {
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
            let mut buf = path.join(Self::get_subdir());
            buf.push(format!("{}.svg", self.id));
            grapher.create_faith_graph(self.id, &buf);
        }
        if let Some(map) = data.get_map() {
            let filter = |title: Ref<Title>| {
                let key = title.get_key();
                if key.is_none() {
                    return false;
                }
                if key.as_ref().unwrap().starts_with("c_") {
                    if let Some(c_faith) = title.get_faith() {
                        return c_faith.get_internal().id == self.id;
                    }
                }
                return false;
            };
            let keys = game_state.get_baronies_of_counties(filter);
            if !keys.is_empty() {
                let mut buf = path.join(Self::get_subdir());
                buf.push(format!("{}.png", self.id));
                let mut faith_map = map.create_map_flat(keys, [70, 255, 70]);
                faith_map.draw_text(format!("Map of the {} faith", &self.name.as_ref().unwrap()));
                faith_map.save(&buf);
            }
        }
    }
}

impl Localizable for Faith {
    fn localize<L: Localize<GameString>>(
        &mut self,
        localization: &mut L,
    ) -> Result<(), LocalizationError> {
        self.name = localization.localize(self.name)?;
        for tenet in self.tenets.iter_mut() {
            *tenet = localization.localize(tenet.to_string() + "_name")?;
        }
        for doctrine in self.doctrines.iter_mut() {
            *doctrine = localization.localize(doctrine.to_string() + "_name")?;
        }
        Ok(())
    }
}
