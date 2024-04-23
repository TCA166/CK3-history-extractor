use std::rc::Rc;
use minijinja::{Environment, context};

use serde::Serialize;
use serde::ser::SerializeStruct;
use super::renderer::Renderable;
use super::{Character, GameObjectDerived};

pub struct Dynasty{
    pub parent: Option<Rc<Dynasty>>,
    pub name: Rc<String>,
    pub members: u32,
    pub houses: u32,
    pub prestigeTot: u32,
    pub prestige: u32,
    pub perks: Vec<Rc<String>>,
    pub leaders: Vec<Rc<Character>>,
}

impl GameObjectDerived for Dynasty {
    fn from_game_object(base:&'_ crate::game_object::GameObject, game_state:&crate::game_state::GameState) -> Self {
        let parent_id = base.get("dynasty").unwrap().as_string().unwrap();
        let currency = base.get("prestige").unwrap().as_object().unwrap();
        let mut perks = Vec::new();
        for p in base.get("perks").unwrap().as_array().unwrap(){
            perks.push(p.as_string().unwrap());
        }
        let mut leaders = Vec::new();
        for l in base.get("leaders").unwrap().as_array().unwrap(){
            leaders.push(Rc::from(game_state.get_character(l.as_string().unwrap().as_str()).unwrap().clone()));
        }
        Dynasty{
            name: base.get("name").unwrap().as_string().unwrap(),
            parent: match parent_id.as_str() {
                "0" => None,
                _ => Some(Rc::from(game_state.get_dynasty(parent_id.as_str()).unwrap().clone()))
            },
            members: 0,
            houses: 0,
            prestigeTot: currency.get("accumulated").unwrap().as_string().unwrap().parse::<u32>().unwrap(),
            prestige: currency.get("currency").unwrap().as_string().unwrap().parse::<u32>().unwrap(),
            perks: perks,
            leaders: leaders
        }
    }

    fn type_name() -> &'static str {
        "dynasty"
    }
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
