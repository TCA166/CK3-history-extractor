use serde::Serialize;

use super::{
    super::{
        display::{Cullable, Grapher, Renderable, RenderableType},
        game_data::{GameData, Localizable, Localize, MapGenerator},
        jinja_env::FAITH_TEMPLATE_NAME,
        parser::{GameId, GameObjectMap, GameState, GameString, ParsingError},
        types::{RefOrRaw, Wrapper, WrapperMut},
    },
    serialize_ref, Character, DummyInit, GameObjectDerived, Shared, Title,
};

/// A struct representing a faith in the game
#[derive(Serialize)]
pub struct Faith {
    id: GameId,
    name: Option<GameString>,
    tenets: Vec<GameString>,
    #[serde(serialize_with = "serialize_ref")]
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
        data: &GameData,
    ) {
        if let Some(grapher) = grapher {
            let path = format!("{}/{}/{}.svg", path, Self::get_subdir(), self.id);
            grapher.create_faith_graph(self.id, &path);
        }
        if let Some(map) = data.get_map() {
            let filter = |title: &RefOrRaw<Title>| {
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
    fn localize<L: Localize>(&mut self, localization: &mut L) {
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
