use std::collections::HashMap;
use std::rc::Rc;

use minijinja::{Environment, context};

use serde::Serialize;
use serde::ser::SerializeStruct;

use crate::game_object::GameObject;

use super::renderer::Renderable;
use super::{Character, GameObjectDerived};

pub struct Title {
    pub name: Rc<String>,
    pub deJure: Rc<Title>,
    pub deFacto: Rc<Title>,
    pub vassals: Vec<Rc<Title>>,
    pub history: HashMap<Rc<String>, (Rc<Character>, Rc<String>)>
}

impl Serialize for Title {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Title", 5)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("deJure", &self.deJure)?;
        state.serialize_field("deFacto", &self.deFacto)?;
        state.serialize_field("vassals", &self.vassals)?;
        state.serialize_field("history", &self.history)?;
        state.end()
    }
}

impl Renderable for Title {
    fn render(&self, env: &Environment, template_name: &'static String) -> String {
        let ctx = context! {title=>self};
        env.get_template(template_name).unwrap().render(&ctx).unwrap()   
    }
}
