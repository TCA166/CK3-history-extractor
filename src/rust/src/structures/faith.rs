use minijinja::{Environment, context};

use serde::Serialize;
use serde::ser::SerializeStruct;

use super::character::Character;

use super::renderer::Renderable;

pub struct Faith<'a> {
    name: &'a String,
    tenets: Vec<&'a String>,
    head: &'a Character<'a>,
    fervor: f32,
    doctrines: Vec<&'a String>,
}

impl Serialize for Faith<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Faith", 5)?;
        state.serialize_field("name", self.name)?;
        state.serialize_field("tenets", &self.tenets)?;
        state.serialize_field("head", self.head)?;
        state.serialize_field("fervor", &self.fervor)?;
        state.serialize_field("doctrines", &self.doctrines)?;
        state.end()
    }
}

impl Renderable for Faith<'_> {
    fn render(&self, env: &Environment, template_name: &'static String) -> String {
        let ctx = context! {faith=>self};
        env.get_template(template_name).unwrap().render(&ctx).unwrap()   
    }
}

