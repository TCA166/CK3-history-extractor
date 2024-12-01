use serde::{ser::SerializeStruct, Serialize};

use super::{
    super::{
        display::{Cullable, GameMap, Grapher, Localizable, Localizer, Renderable, RenderableType},
        jinja_env::FAITH_TEMPLATE_NAME,
        parser::{GameId, GameObjectArray, GameObjectMap, GameState, GameString},
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

/// Gets the head of the faith
fn get_head(base: &GameObjectMap, game_state: &mut GameState) -> Option<Shared<Character>> {
    if let Some(current) = base.get("religious_head") {
        let title = game_state.get_title(&current.as_id());
        return title.get_internal().get_holder();
    }
    None
}

/// Gets the tenets of the faith and appends them to the tenets vector
fn get_tenets(tenets: &mut Vec<GameString>, array: &GameObjectArray) {
    for t in array {
        let s = t.as_string();
        if s.contains("tenet") {
            tenets.push(s);
        }
    }
}

/// Gets the doctrines of the faith and appends them to the doctrines vector
fn get_doctrines(doctrines: &mut Vec<GameString>, array: &GameObjectArray) {
    for d in array {
        let s = d.as_string();
        if !s.contains("tenet") {
            doctrines.push(s);
        }
    }
}

/// Gets the name of the faith
fn get_name(base: &GameObjectMap) -> GameString {
    if let Some(node) = base.get("name") {
        return node.as_string();
    } else {
        base.get("template").unwrap().as_string()
    }
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

    fn init(&mut self, base: &GameObjectMap, game_state: &mut GameState) {
        let doctrines_array = base.get("doctrine").unwrap().as_object().as_array();
        get_tenets(&mut self.tenets, doctrines_array);
        self.head.clone_from(&get_head(&base, game_state));
        get_doctrines(&mut self.doctrines, doctrines_array);
        self.name = Some(get_name(&base));
        self.fervor = base
            .get("fervor")
            .unwrap()
            .as_string()
            .parse::<f32>()
            .unwrap();
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
