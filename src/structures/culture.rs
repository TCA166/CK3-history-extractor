use minijinja::context;
use serde::Serialize;
use serde::ser::SerializeStruct;
use super::renderer::Renderable;
use super::{Cullable, GameObjectDerived, Shared, serialize_array};
use crate::game_object::GameObject;
use std::rc::Rc;

/// A struct representing a culture in the game
pub struct Culture {
    id: u32,
    name: Rc<String>,
    ethos: Rc<String>,
    heritage: Rc<String>,
    martial: Rc<String>,
    date: Option<Rc<String>>,
    parents: Vec<Shared<Culture>>,
    traditions: Vec<Rc<String>>,
    language: Rc<String>,
    depth: usize
}

/// Gets the parents of the culture and appends them to the parents vector
fn get_parents(parents:&mut Vec<Shared<Culture>>, base:&GameObject, game_state:&mut crate::game_state::GameState){
    let parents_obj = base.get("parents");
    if parents_obj.is_some(){
        for p in parents_obj.unwrap().as_object().unwrap().get_array_iter(){
            parents.push(game_state.get_culture(p.as_string().as_str()).clone());
        }
    }
}

/// Gets the traditions of the culture and appends them to the traditions vector
fn get_traditions(traditions:&mut Vec<Rc<String>>, base:&&GameObject){
    let traditions_obj = base.get("traditions");
    if traditions_obj.is_some(){
        for t in traditions_obj.unwrap().as_object().unwrap().get_array_iter(){
            traditions.push(t.as_string());
        }
    }
}

/// Gets the creation date of the culture
fn get_date(base:&GameObject) -> Option<Rc<String>>{
    let node = base.get("created");
    if node.is_some(){
        return Some(node.unwrap().as_string());
    }
    None
}

impl GameObjectDerived for Culture {
    fn from_game_object(base:&GameObject, game_state:&mut crate::game_state::GameState) -> Self {
        let mut parents = Vec::new();
        get_parents(&mut parents, base, game_state);
        let mut traditions = Vec::new();
        get_traditions(&mut traditions, &base);
        Culture{
            name: base.get("name").unwrap().as_string(),
            ethos: base.get("ethos").unwrap().as_string(),
            heritage: base.get("heritage").unwrap().as_string(),
            martial: base.get("martial_custom").unwrap().as_string(),
            date: get_date(&base),
            parents: parents,
            traditions: traditions,
            id: base.get_name().parse::<u32>().unwrap(),
            language: base.get("language").unwrap().as_string(),
            depth: 0
        }
    }

    fn dummy(id:u32) -> Self {
        Culture{
            name: Rc::new("".to_owned().into()),
            ethos: Rc::new("".to_owned().into()),
            heritage: Rc::new("".to_owned().into()),
            martial: Rc::new("".to_owned().into()),
            date: None,
            parents: Vec::new(),
            traditions: Vec::new(),
            language: Rc::new("".to_owned().into()),
            id: id,
            depth: 0
        }
    }

    fn init(&mut self, base: &GameObject, game_state: &mut crate::game_state::GameState) {
        get_parents(&mut self.parents, &base, game_state);
        get_traditions(&mut self.traditions, &base);
        self.name = base.get("name").unwrap().as_string();
        self.ethos = base.get("ethos").unwrap().as_string();
        self.heritage = base.get("heritage").unwrap().as_string();
        self.martial = base.get("martial_custom").unwrap().as_string();
        self.date = get_date(&base);
        self.language = base.get("language").unwrap().as_string();
    }

    fn get_id(&self) -> u32 {
        self.id
    }

    fn get_name(&self) -> Rc<String> {
        self.name.clone()
    }
}

impl Serialize for Culture {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Culture", 8)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("ethos", &self.ethos)?;
        state.serialize_field("heritage", &self.heritage)?;
        state.serialize_field("martial", &self.martial)?;
        state.serialize_field("date", &self.date)?;
        let parents = serialize_array(&self.parents);
        state.serialize_field("parents", &parents)?;
        state.serialize_field("traditions", &self.traditions)?;
        state.serialize_field("language", &self.language)?;
        state.end()
    }
}

impl Renderable for Culture {
    fn get_context(&self) -> minijinja::Value {
        context!{culture=>self}
    }

    fn get_template() -> &'static str {
        "cultureTemplate.html"
    }

    fn get_subdir() -> &'static str {
        "cultures"
    }

    fn render_all(&self, renderer: &mut super::Renderer) {
        if !renderer.render(self){
            return;
        }
        for p in &self.parents{
            p.borrow().render_all(renderer);
        }
    }
}

impl Cullable for Culture {
    fn get_depth(&self) -> usize {
        self.depth
    }

    fn set_depth(&mut self, depth:usize) {
        if depth <= self.depth || depth == 0{
            return;
        }
        self.depth = depth;
        for p in &self.parents{
            p.borrow_mut().set_depth(depth-1);
        }
    }
}
