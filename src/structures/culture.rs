use minijinja::{Environment, context};
use serde::Serialize;
use serde::ser::SerializeStruct;
use super::renderer::Renderable;
use super::{Cullable, GameObjectDerived, Shared};
use crate::game_object::GameObject;
use std::cell::Ref;

pub struct Culture {
    pub id: u32,
    pub name: Shared<String>,
    pub ethos: Shared<String>,
    pub heritage: Shared<String>,
    pub martial: Shared<String>,
    pub date: Option<Shared<String>>,
    pub parents: Vec<Shared<Culture>>,
    pub traditions: Vec<Shared<String>>,
    depth: usize
}

fn get_parents(parents:&mut Vec<Shared<Culture>>, base:&Ref<'_, GameObject>, game_state:&mut crate::game_state::GameState){
    let parents_obj = base.get("parents");
    if parents_obj.is_some(){
        for p in parents_obj.unwrap().as_object_ref().unwrap().get_array_iter(){
            parents.push(game_state.get_culture(p.as_string_ref().unwrap().as_str()).clone());
        }
    }
}

fn get_traditions(traditions:&mut Vec<Shared<String>>, base:&Ref<'_, GameObject>){
    let traditions_obj = base.get("traditions");
    if traditions_obj.is_some(){
        for t in traditions_obj.unwrap().as_object_ref().unwrap().get_array_iter(){
            traditions.push(t.as_string());
        }
    }
}

fn get_date(base:&Ref<'_, GameObject>) -> Option<Shared<String>>{
    let node = base.get("date");
    if node.is_some(){
        return Some(node.unwrap().as_string());
    }
    None
}

impl GameObjectDerived for Culture {
    fn from_game_object(base:Ref<'_, GameObject>, game_state:&mut crate::game_state::GameState) -> Self {
        let mut parents = Vec::new();
        get_parents(&mut parents, &base, game_state);
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
            depth: 0
        }
    }

    fn dummy(id:u32) -> Self {
        Culture{
            name: Shared::new("".to_owned().into()),
            ethos: Shared::new("".to_owned().into()),
            heritage: Shared::new("".to_owned().into()),
            martial: Shared::new("".to_owned().into()),
            date: None,
            parents: Vec::new(),
            traditions: Vec::new(),
            id: id,
            depth: 0
        }
    }

    fn init(&mut self, base: Ref<'_, GameObject>, game_state: &mut crate::game_state::GameState) {
        get_parents(&mut self.parents, &base, game_state);
        get_traditions(&mut self.traditions, &base);
        self.name = base.get("name").unwrap().as_string();
        self.ethos = base.get("ethos").unwrap().as_string();
        self.heritage = base.get("heritage").unwrap().as_string();
        self.martial = base.get("martial_custom").unwrap().as_string();
        self.date = get_date(&base);
    }

    fn get_id(&self) -> u32 {
        self.id
    }
}

impl Serialize for Culture {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Culture", 7)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("ethos", &self.ethos)?;
        state.serialize_field("heritage", &self.heritage)?;
        state.serialize_field("martial", &self.martial)?;
        state.serialize_field("date", &self.date)?;
        state.serialize_field("parents", &self.parents)?;
        state.serialize_field("traditions", &self.traditions)?;
        state.end()
    }
}

impl Renderable for Culture {
    fn render(&self, env: &Environment) -> String {
        let ctx = context! {culture=>self};
        env.get_template("cultureTemplate.html").unwrap().render(&ctx).unwrap()   
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
