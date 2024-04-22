use std::collections::HashMap;

use minijinja::{Environment, context};

use serde::Serialize;
use serde::ser::SerializeStruct;

use super::renderer::Renderable;
use super::Character;

pub struct Title<'a> {
    name: &'a String,
    deJure: &'a Title<'a>,
    deFacto: &'a Title<'a>,
    vassals: Vec<&'a Title<'a>>,
    history: HashMap<&'a String, (&'a Character<'a>, &'a String)>
}

impl Serialize for Title<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Title", 5)?;
        state.serialize_field("name", self.name)?;
        state.serialize_field("deJure", self.deJure)?;
        state.serialize_field("deFacto", self.deFacto)?;
        state.serialize_field("vassals", &self.vassals)?;
        state.serialize_field("history", &self.history)?;
        state.end()
    }
}

impl Renderable for Title<'_> {
    fn render(&self, env: &Environment, template_name: &'static String) -> String {
        let ctx = context! {title=>self};
        env.get_template(template_name).unwrap().render(&ctx).unwrap()   
    }
}
