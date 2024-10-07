use serde::{ser::SerializeStruct, Serialize};

use super::{
    super::{
        display::{
            Cullable, Localizable, Localizer, Renderable, RenderableType, Renderer, TreeNode,
        },
        jinja_env::CUL_TEMPLATE_NAME,
        parser::{GameId, GameObjectMap, GameState, GameString},
        types::Wrapper,
    },
    serialize_array, DummyInit, GameObjectDerived, Shared,
};

/// A struct representing a culture in the game
pub struct Culture {
    id: GameId,
    name: Option<GameString>,
    ethos: Option<GameString>,
    heritage: Option<GameString>,
    martial: Option<GameString>,
    date: Option<GameString>,
    children: Vec<Shared<Culture>>,
    parents: Vec<Shared<Culture>>,
    traditions: Vec<GameString>,
    language: Option<GameString>,
    depth: usize,
}

/// Gets the parents of the culture and appends them to the parents vector
fn get_parents(
    parents: &mut Vec<Shared<Culture>>,
    base: &GameObjectMap,
    id: GameId,
    game_state: &mut GameState,
) {
    if let Some(parents_obj) = base.get("parents") {
        for p in parents_obj.as_object().as_array() {
            let parent = game_state.get_culture(&p.as_id()).clone();
            if let Ok(mut r) = parent.try_borrow_mut() {
                r.register_child(game_state.get_culture(&id));
            }
            parents.push(parent.clone());
        }
    }
}

/// Gets the traditions of the culture and appends them to the traditions vector
fn get_traditions(traditions: &mut Vec<GameString>, base: &GameObjectMap) {
    if let Some(traditions_obj) = base.get("traditions") {
        for t in traditions_obj.as_object().as_array() {
            traditions.push(t.as_string());
        }
    }
}

/// Gets the creation date of the culture
fn get_date(base: &GameObjectMap) -> Option<GameString> {
    if let Some(node) = base.get("created") {
        return Some(node.as_string());
    }
    None
}

impl DummyInit for Culture {
    fn dummy(id: GameId) -> Self {
        Culture {
            name: None,
            ethos: None,
            heritage: None,
            martial: None,
            date: None,
            parents: Vec::new(),
            children: Vec::new(),
            traditions: Vec::new(),
            language: None,
            id: id,
            depth: 0,
        }
    }

    fn init(&mut self, base: &GameObjectMap, game_state: &mut GameState) {
        get_parents(&mut self.parents, &base, self.id, game_state);
        get_traditions(&mut self.traditions, &base);
        self.name = Some(base.get("name").unwrap().as_string());
        if let Some(eth) = base.get("ethos") {
            //this is possible, shoutout u/Kinc4id
            self.ethos = Some(eth.as_string());
        }
        self.heritage = Some(base.get("heritage").unwrap().as_string());
        self.martial = Some(base.get("martial_custom").unwrap().as_string());
        self.date = get_date(&base);
        self.language = Some(base.get("language").unwrap().as_string());
    }
}

impl GameObjectDerived for Culture {
    fn get_id(&self) -> GameId {
        self.id
    }

    fn get_name(&self) -> GameString {
        self.name.as_ref().unwrap().clone()
    }
}

impl TreeNode for Culture {
    fn get_children(&self) -> &Vec<Shared<Self>> {
        &self.children
    }

    fn get_parent(&self) -> &Vec<Shared<Self>> {
        &self.parents
    }

    fn get_class(&self) -> Option<GameString> {
        if let Some(heritage) = &self.heritage {
            return Some(heritage.clone());
        } else {
            return None;
        }
    }
}

impl Culture {
    pub fn register_child(&mut self, child: Shared<Culture>) {
        self.children.push(child);
    }
}

impl Serialize for Culture {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Culture", 10)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("ethos", &self.ethos)?;
        state.serialize_field("heritage", &self.heritage)?;
        state.serialize_field("martial", &self.martial)?;
        state.serialize_field("date", &self.date)?;
        let parents = serialize_array(&self.parents);
        state.serialize_field("parents", &parents)?;
        let children = serialize_array(&self.children);
        state.serialize_field("children", &children)?;
        state.serialize_field("traditions", &self.traditions)?;
        state.serialize_field("language", &self.language)?;
        state.end()
    }
}

impl Renderable for Culture {
    fn get_template() -> &'static str {
        CUL_TEMPLATE_NAME
    }

    fn get_subdir() -> &'static str {
        "cultures"
    }

    fn render_all(&self, stack: &mut Vec<RenderableType>, renderer: &mut Renderer) {
        if !renderer.render(self) {
            return;
        }
        if let Some(grapher) = renderer.get_grapher() {
            let path = format!(
                "{}/{}/{}.svg",
                renderer.get_path(),
                Self::get_subdir(),
                self.id
            );
            grapher.create_culture_graph(self.id, &path);
        }
        if let Some(map) = renderer.get_map() {
            let game_state = renderer.get_state();
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
                let c_culture = title.get_culture().unwrap();
                if c_culture.get_internal().id == self.id {
                    keys.append(&mut title.get_barony_keys());
                }
            }
            if !keys.is_empty() {
                let path = format!(
                    "{}/{}/{}.png",
                    renderer.get_path(),
                    Self::get_subdir(),
                    self.id
                );
                map.create_map_file(
                    keys,
                    &[70, 255, 70],
                    &path,
                    &format!("Map of the {} culture", &self.name.as_ref().unwrap()),
                );
            }
        }
        for p in &self.parents {
            stack.push(RenderableType::Culture(p.clone()));
        }
        for c in &self.children {
            stack.push(RenderableType::Culture(c.clone()));
        }
    }
}

impl Localizable for Culture {
    fn localize(&mut self, localization: &Localizer) {
        self.name = Some(localization.localize(self.name.as_ref().unwrap().as_str()));
        self.ethos = Some(localization.localize(self.ethos.as_ref().unwrap().as_str()));
        self.heritage = Some(localization.localize(self.heritage.as_ref().unwrap().as_str()));
        self.martial = Some(localization.localize(self.martial.as_ref().unwrap().as_str()));
        self.language = Some(localization.localize(self.language.as_ref().unwrap().as_str()));
        for t in &mut self.traditions {
            *t = localization.localize(t.as_str());
        }
    }
}

impl Cullable for Culture {
    fn get_depth(&self) -> usize {
        self.depth
    }

    fn set_depth(&mut self, depth: usize) {
        if depth <= self.depth {
            return;
        }
        self.depth = depth;
        for p in &self.parents {
            if let Ok(mut r) = p.try_borrow_mut() {
                r.set_depth(depth);
            }
        }
        for c in &self.children {
            if let Ok(mut r) = c.try_borrow_mut() {
                r.set_depth(depth);
            }
        }
    }
}
