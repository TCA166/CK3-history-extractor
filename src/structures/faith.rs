use std::{cell::Ref, path::Path};

use serde::Serialize;

use super::{
    super::{
        display::{Grapher, Renderable},
        game_data::{GameData, Localizable, LocalizationError, Localize, MapGenerator},
        jinja_env::FAITH_TEMPLATE_NAME,
        parser::{GameId, GameObjectMap, GameObjectMapping, GameState, GameString, ParsingError},
        types::Wrapper,
    },
    Character, DummyInit, GameObjectDerived, GameObjectDerivedType, Shared, Title,
};

/// A struct representing a faith in the game
#[derive(Serialize, Debug)]
pub struct Faith {
    id: GameId,
    name: Option<GameString>,
    tenets: Vec<GameString>,
    head: Option<Shared<Character>>,
    fervor: f32,
    doctrines: Vec<GameString>,
}

impl DummyInit for Faith {
    fn dummy(id: GameId) -> Self {
        Faith {
            name: None,
            tenets: Vec::new(),
            head: None, //trying to create a dummy character here caused a fascinating stack overflow because of infinite recursion
            fervor: 0.0,
            doctrines: Vec::new(),
            id: id,
        }
    }

    fn init(
        &mut self,
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<(), ParsingError> {
        let doctrines_array = base.get_object("doctrine")?.as_array()?;
        for t in doctrines_array {
            let s = t.as_string()?;
            if s.contains("tenet") {
                self.tenets.push(s);
            } else {
                self.doctrines.push(s);
            }
        }
        if let Some(current) = base.get("religious_head") {
            let title = game_state.get_title(&current.as_id()?);
            self.head = title.get_internal().get_holder();
        }
        if let Some(node) = base.get("name") {
            self.name = Some(node.as_string()?);
        } else {
            self.name = Some(base.get_string("template")?);
        }
        self.fervor = base.get_real("fervor")? as f32;
        Ok(())
    }
}

impl GameObjectDerived for Faith {
    fn get_id(&self) -> GameId {
        self.id
    }

    fn get_name(&self) -> Option<GameString> {
        self.name.as_ref().map(|x| x.clone())
    }

    fn get_references<E: From<GameObjectDerivedType>, C: Extend<E>>(&self, collection: &mut C) {
        if let Some(head) = &self.head {
            collection.extend([E::from(head.clone().into())]);
        }
    }
}

impl Renderable for Faith {
    fn get_template() -> &'static str {
        FAITH_TEMPLATE_NAME
    }

    fn get_subdir() -> &'static str {
        "faiths"
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
        self.name = Some(localization.localize(self.name.as_ref().unwrap())?);
        for tenet in self.tenets.iter_mut() {
            *tenet = localization.localize(tenet.to_string() + "_name")?;
        }
        for doctrine in self.doctrines.iter_mut() {
            *doctrine = localization.localize(doctrine.to_string() + "_name")?;
        }
        Ok(())
    }
}
