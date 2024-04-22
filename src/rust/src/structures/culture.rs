use minijinja::{Environment, context};
use std::rc::Rc;
use serde::Serialize;
use serde::ser::SerializeStruct;
use super::renderer::Renderable;

pub struct Culture {
    name: Rc<String>,
    ethos: Rc<String>,
    heritage: Rc<String>,
    martial: Rc<String>,
    date: Rc<String>,
    parents: Vec<Rc<Culture>>,
    traditions: Vec<Rc<String>>,
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
