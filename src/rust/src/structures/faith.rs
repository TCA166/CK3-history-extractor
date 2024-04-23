use minijinja::{Environment, context};
use std::rc::Rc;
use serde::Serialize;
use serde::ser::SerializeStruct;
use super::{Character, GameObjectDerived};
use super::renderer::Renderable;

pub struct Faith {
    pub name: Rc<String>,
    pub tenets: Vec<Rc<String>>,
    pub head: Rc<Character>,
    pub fervor: f32,
    pub doctrines: Vec<Rc<String>>,
}

impl GameObjectDerived for Faith {
    fn from_game_object(base:&'_ crate::game_object::GameObject, game_state:&crate::game_state::GameState) -> Self {
        let mut tenets = Vec::new();
        for t in base.get("tenets").unwrap().as_array().unwrap(){
            tenets.push(t.as_string().unwrap());
        }
        let head = Rc::from(game_state.get_character(base.get("head").unwrap().as_string().unwrap().as_str()).unwrap().clone());
        let mut doctrines = Vec::new();
        for d in base.get("doctrines").unwrap().as_array().unwrap(){
            doctrines.push(d.as_string().unwrap());
        }
        Faith{
            name: base.get("name").unwrap().as_string().unwrap(),
            tenets: tenets,
            head: head,
            fervor: base.get("fervor").unwrap().as_string().unwrap().parse::<f32>().unwrap(),
            doctrines: doctrines
        }
    }

    fn type_name() -> &'static str {
        "faith"
    }
}

impl Serialize for Faith {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Faith", 5)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("tenets", &self.tenets)?;
        state.serialize_field("head", &self.head)?;
        state.serialize_field("fervor", &self.fervor)?;
        state.serialize_field("doctrines", &self.doctrines)?;
        state.end()
    }
}

impl Renderable for Faith {
    fn render(&self, env: &Environment, template_name: &'static String) -> String {
        let ctx = context! {faith=>self};
        env.get_template(template_name).unwrap().render(&ctx).unwrap()   
    }
}
