use minijinja::{Environment, context};
use std::rc::Rc;
use serde::Serialize;
use serde::ser::SerializeStruct;
use super::renderer::Renderable;
use super::GameObjectDerived;

pub struct Culture {
    pub name: Rc<String>,
    pub ethos: Rc<String>,
    pub heritage: Rc<String>,
    pub martial: Rc<String>,
    pub date: Rc<String>,
    pub parents: Vec<Rc<Culture>>,
    pub traditions: Vec<Rc<String>>,
}

impl GameObjectDerived for Culture {
    fn from_game_object(base:&'_ crate::game_object::GameObject, game_state:&crate::game_state::GameState) -> Self {
        let mut parents = Vec::new();
        for p in base.get("parents").unwrap().as_array().unwrap(){
            parents.push(Rc::from(game_state.get_culture(p.as_string().unwrap().as_str()).unwrap().clone()));
        }
        let mut traditions = Vec::new();
        for t in base.get("traditions").unwrap().as_array().unwrap(){
            traditions.push(t.as_string().unwrap());
        }
        Culture{
            name: base.get("name").unwrap().as_string().unwrap(),
            ethos: base.get("ethos").unwrap().as_string().unwrap(),
            heritage: base.get("heritage").unwrap().as_string().unwrap(),
            martial: base.get("martial").unwrap().as_string().unwrap(),
            date: base.get("date").unwrap().as_string().unwrap(),
            parents: parents,
            traditions: traditions
        }
    }

    fn type_name() -> &'static str {
        "culture"
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
