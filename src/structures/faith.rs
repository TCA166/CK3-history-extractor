use serde::{ser::SerializeStruct, Serialize};

use super::{
    super::{
        display::{Cullable, GameMap, Grapher, Localizable, Localizer, Renderable, RenderableType},
        jinja_env::FAITH_TEMPLATE_NAME,
        parser::{GameId, GameObjectMap, GameState, GameString, ParsingError},
        types::{Wrapper, WrapperMut},
    },
    Character, DerivedRef, DummyInit, GameObjectDerived, Shared,
};

/// A struct representing a faith in the game
pub struct Faith {
    id: GameId,
    name: Option<GameString>,
    tenets: Vec<GameString>,
    head: Option<Shared<Character>>,
    fervor: f32,
    doctrines: Vec<GameString>,
    depth: usize,
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
            depth: 0,
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

    fn get_name(&self) -> GameString {
        self.name.as_ref().unwrap().clone()
    }
}

impl Serialize for Faith {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Faith", 6)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("tenets", &self.tenets)?;
        if let Some(head) = &self.head {
            let head = DerivedRef::<Character>::from_derived(head.clone());
            state.serialize_field("head", &head)?;
        }
        state.serialize_field("fervor", &self.fervor)?;
        state.serialize_field("doctrines", &self.doctrines)?;
        state.end()
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
        path: &str,
        game_state: &GameState,
        grapher: Option<&Grapher>,
        map: Option<&GameMap>,
    ) {
        if let Some(grapher) = grapher {
            let path = format!("{}/{}/{}.svg", path, Self::get_subdir(), self.id);
            grapher.create_faith_graph(self.id, &path);
        }
        if let Some(map) = map {
            let mut keys = Vec::new();
            for entry in game_state.get_title_iter() {
                let title = entry.1.get_internal();
                let key = title.get_key();
                if key.is_none() {
                    continue;
                }
                if !key.as_ref().unwrap().starts_with("c_") {
                    continue;
                }
                let c_faith = title.get_faith().unwrap();
                if c_faith.get_internal().id == self.id {
                    keys.append(&mut title.get_barony_keys());
                }
            }
            if !keys.is_empty() {
                let path = format!("{}/{}/{}.png", path, Self::get_subdir(), self.id);
                map.create_map_file(
                    keys,
                    &[70, 255, 70],
                    &path,
                    &format!("Map of the {} faith", &self.name.as_ref().unwrap()),
                );
            }
        }
    }

    fn append_ref(&self, stack: &mut Vec<RenderableType>) {
        if let Some(head) = &self.head {
            stack.push(RenderableType::Character(head.clone()));
        }
    }
}

impl Localizable for Faith {
    fn localize(&mut self, localization: &mut Localizer) {
        self.name = Some(localization.localize(self.name.as_ref().unwrap().as_str()));
        for tenet in self.tenets.iter_mut() {
            *tenet = localization.localize(tenet.as_str());
        }
        for doctrine in self.doctrines.iter_mut() {
            *doctrine = localization.localize(doctrine.as_str());
        }
    }
}

impl Cullable for Faith {
    fn get_depth(&self) -> usize {
        self.depth
    }

    fn set_depth(&mut self, depth: usize) {
        if depth <= self.depth {
            return;
        }
        self.depth = depth;
        if let Some(head) = &self.head {
            if let Ok(mut head) = head.try_get_internal_mut() {
                head.set_depth(depth - 1);
            }
        }
    }
}
