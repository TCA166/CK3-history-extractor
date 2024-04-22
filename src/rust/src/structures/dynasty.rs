use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use minijinja::{Environment, context};
use serde::Serialize;
use serde::ser::SerializeStruct;
use super::renderer::Renderable;
use super::Character;

pub struct Dynasty{
    parent: Rc<RefCell<Dynasty>>,
    name: Rc<String>,
    members: u32,
    houses: u32,
    prestigeTot: u32,
    prestige: u32,
    perks: HashMap<Rc<String>, u32>,
    leaders: Vec<Rc<Character>>,
}

impl Serialize for Dynasty {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Dynasty", 8)?;
        state.serialize_field("parent", &self.parent)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("members", &self.members)?;
        state.serialize_field("houses", &self.houses)?;
        state.serialize_field("prestigeTot", &self.prestigeTot)?;
        state.serialize_field("prestige", &self.prestige)?;
        state.serialize_field("perks", &self.perks)?;
        state.serialize_field("leaders", &self.leaders)?;
        state.end()
    }
}

impl Renderable for Dynasty {
    fn render(&self, env: &Environment, template_name: &'static String) -> String {
        let ctx = context! {dynasty=>self};
        env.get_template(template_name).unwrap().render(&ctx).unwrap()   
    }
}
