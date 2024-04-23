use minijinja::{Environment, context};

use serde::Serialize;
use serde::ser::SerializeStruct;
use super::renderer::Renderable;
use super::{Character, GameObjectDerived, Shared};
use crate::game_object::GameObject;
use std::cell::Ref;

pub struct Dynasty{
    pub parent: Option<Shared<Dynasty>>,
    pub name: Shared<String>,
    pub members: u32,
    pub houses: u32,
    pub prestigeTot: u32,
    pub prestige: u32,
    pub perks: Vec<Shared<String>>,
    pub leaders: Vec<Shared<Character>>,
}

impl GameObjectDerived for Dynasty {
    fn from_game_object(base:Ref<'_, GameObject>, game_state:&crate::game_state::GameState) -> Self {
        let parent_id = base.get_string_ref("dynasty");
        let currency = base.get_object_ref("prestige");
        let mut perks = Vec::new();
        for p in base.get_object_ref("perks").get_array(){
            perks.push(p.as_string());
        }
        let mut leaders = Vec::new();
        for l in base.get_object_ref("leaders").get_array(){
            leaders.push(game_state.get_character(l.as_string_ref().unwrap().as_str()).unwrap().clone());
        }
        let prestige_tot = currency.get_string_ref("accumulated").parse::<u32>().unwrap();
        let prestige = currency.get_string_ref("currency").parse::<u32>().unwrap();
        Dynasty{
            name: base.get("name").unwrap().as_string(),
            parent: match parent_id.as_str() {
                "0" => None,
                _ => Some(game_state.get_dynasty(parent_id.as_str()).unwrap().clone())
            },
            members: 0,
            houses: 0,
            prestigeTot: prestige_tot,
            prestige: prestige,
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
