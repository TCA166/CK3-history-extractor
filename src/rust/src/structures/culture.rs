use minijinja::{Environment, context};
use serde::Serialize;
use serde::ser::SerializeStruct;
use super::renderer::Renderable;
use super::{GameObjectDerived, Shared};
use crate::game_object::GameObject;
use std::cell::Ref;

pub struct Culture {
    pub name: Shared<String>,
    pub ethos: Shared<String>,
    pub heritage: Shared<String>,
    pub martial: Shared<String>,
    pub date: Shared<String>,
    pub parents: Vec<Shared<Culture>>,
    pub traditions: Vec<Shared<String>>,
}

impl GameObjectDerived for Culture {
    fn from_game_object(base:Ref<'_, GameObject>, game_state:&mut crate::game_state::GameState) -> Self {
        let mut parents = Vec::new();
        for p in base.get_object_ref("parents").get_array(){
            parents.push(game_state.get_culture(p.as_string_ref().unwrap().as_str()).clone());
        }
        let mut traditions = Vec::new();
        for t in base.get_object_ref("traditions").get_array(){
            traditions.push(t.as_string());
        }
        Culture{
            name: base.get("name").unwrap().as_string(),
            ethos: base.get("ethos").unwrap().as_string(),
            heritage: base.get("heritage").unwrap().as_string(),
            martial: base.get("martial").unwrap().as_string(),
            date: base.get("date").unwrap().as_string(),
            parents: parents,
            traditions: traditions
        }
    }

    fn type_name() -> &'static str {
        "culture"
    }

    fn dummy() -> Self {
        Culture{
            name: Shared::new("".to_owned().into()),
            ethos: Shared::new("".to_owned().into()),
            heritage: Shared::new("".to_owned().into()),
            martial: Shared::new("".to_owned().into()),
            date: Shared::new("".to_owned().into()),
            parents: Vec::new(),
            traditions: Vec::new()
        }
    }

    fn init(&mut self, base: Ref<'_, GameObject>, game_state: &mut crate::game_state::GameState) {
        let mut parents = Vec::new();
        for p in base.get_object_ref("parents").get_array(){
            parents.push(game_state.get_culture(p.as_string_ref().unwrap().as_str()).clone());
        }
        self.parents = parents;
        let mut traditions = Vec::new();
        for t in base.get_object_ref("traditions").get_array(){
            traditions.push(t.as_string());
        }
        self.traditions = traditions;
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
    fn render(&self, env: &Environment, template_name: &'static String) -> String {
        let ctx = context! {culture=>self};
        env.get_template(template_name).unwrap().render(&ctx).unwrap()   
    }
}
