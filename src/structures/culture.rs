use minijinja::context;

use serde::{ser::SerializeStruct, Serialize};

use super::super::{
    display::{Cullable, Localizer, Renderable, RenderableType, Renderer, TreeNode},
    game_object::{GameObject, GameString},
    game_state::GameState,
};

use super::{serialize_array, DummyInit, GameId, GameObjectDerived, Shared};

use std::slice::Iter;

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
    localized: bool,
    name_localized: bool,
}

/// Gets the parents of the culture and appends them to the parents vector
fn get_parents(
    parents: &mut Vec<Shared<Culture>>,
    base: &GameObject,
    id: GameId,
    game_state: &mut GameState,
) {
    let parents_obj = base.get("parents");
    if parents_obj.is_some() {
        for p in parents_obj.unwrap().as_object().unwrap().get_array_iter() {
            let parent = game_state.get_culture(&p.as_id()).clone();
            let r = parent.try_borrow_mut();
            if r.is_ok() {
                r.unwrap().register_child(game_state.get_culture(&id));
            }
            parents.push(parent.clone());
        }
    }
}

/// Gets the traditions of the culture and appends them to the traditions vector
fn get_traditions(traditions: &mut Vec<GameString>, base: &&GameObject) {
    let traditions_obj = base.get("traditions");
    if traditions_obj.is_some() {
        for t in traditions_obj
            .unwrap()
            .as_object()
            .unwrap()
            .get_array_iter()
        {
            traditions.push(t.as_string());
        }
    }
}

/// Gets the creation date of the culture
fn get_date(base: &GameObject) -> Option<GameString> {
    let node = base.get("created");
    if node.is_some() {
        return Some(node.unwrap().as_string());
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
            localized: false,
            name_localized: false,
        }
    }

    fn init(&mut self, base: &GameObject, game_state: &mut GameState) {
        get_parents(&mut self.parents, &base, self.id, game_state);
        get_traditions(&mut self.traditions, &base);
        self.name = Some(base.get("name").unwrap().as_string());
        let eth = base.get("ethos");
        if eth.is_some() {
            //this is possible, shoutout u/Kinc4id
            self.ethos = Some(eth.unwrap().as_string());
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
    fn get_children_iter(&self) -> Iter<Shared<Self>> {
        self.children.iter()
    }

    fn get_parent_iter(&self) -> Iter<Shared<Self>> {
        self.parents.iter()
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
    fn get_context(&self) -> minijinja::Value {
        context! {culture=>self}
    }

    fn get_template() -> &'static str {
        "cultureTemplate.html"
    }

    fn get_subdir() -> &'static str {
        "cultures"
    }

    fn render_all(&self, stack: &mut Vec<RenderableType>, renderer: &mut Renderer) {
        if !renderer.render(self) {
            return;
        }
        let grapher = renderer.get_grapher();
        if grapher.is_some() {
            let path = format!("{}/cultures/{}.svg", renderer.get_path(), self.id);
            grapher.unwrap().create_culture_graph(self.id, &path);
        }
        for p in &self.parents {
            stack.push(RenderableType::Culture(p.clone()));
        }
        for c in &self.children {
            stack.push(RenderableType::Culture(c.clone()));
        }
    }
}

impl Cullable for Culture {
    fn get_depth(&self) -> usize {
        self.depth
    }

    fn set_depth(&mut self, depth: usize, localization: &Localizer) {
        if depth <= self.depth && depth != 0 {
            return;
        }
        if !self.name_localized {
            self.name = Some(localization.localize(self.name.as_ref().unwrap().as_str()));
            self.name_localized = true;
        }
        if depth == 0 {
            return;
        }
        if !self.localized {
            self.ethos = Some(localization.localize(self.ethos.as_ref().unwrap().as_str()));
            self.heritage = Some(localization.localize(self.heritage.as_ref().unwrap().as_str()));
            self.martial = Some(localization.localize(self.martial.as_ref().unwrap().as_str()));
            self.language = Some(localization.localize(self.language.as_ref().unwrap().as_str()));
            for t in &mut self.traditions {
                *t = localization.localize(t.as_str());
            }
            self.localized = true;
        }
        self.depth = depth;
        for p in &self.parents {
            let r = p.try_borrow_mut();
            if r.is_ok() {
                r.unwrap().set_depth(depth - 1, localization);
            }
        }
        for c in &self.children {
            let r = c.try_borrow_mut();
            if r.is_ok() {
                r.unwrap().set_depth(depth - 1, localization);
            }
        }
    }
}
