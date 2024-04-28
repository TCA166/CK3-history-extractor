use minijinja::{Environment, context};

use serde::Serialize;
use serde::ser::SerializeStruct;
use super::renderer::Renderable;
use super::{Character, GameObjectDerived, Shared};
use crate::game_object::{GameObject, SaveFileValue};
use std::cell::Ref;

pub struct Dynasty{
    pub id: u32,
    pub parent: Option<Shared<Dynasty>>,
    pub name: Option<Shared<String>>,
    pub members: u32,
    pub houses: u32,
    pub prestige_tot: f32,
    pub prestige: f32,
    pub perks: Vec<Shared<String>>,
    pub leaders: Vec<Shared<Character>>,
}

impl GameObjectDerived for Dynasty {
    fn from_game_object(base:Ref<'_, GameObject>, game_state:&mut crate::game_state::GameState) -> Self {
        //get the dynasty legacies
        let mut perks = Vec::new();
        let perks_obj = base.get("perks");
        if perks_obj.is_some(){
            for p in perks_obj.unwrap().as_object_ref().unwrap().get_array_iter(){
                perks.push(p.as_string());
            }
        }
        //get the array of leaders
        let mut leaders = Vec::new();
        let leaders_obj = base.get("historical");
        if perks_obj.is_some(){
            for l in leaders_obj.unwrap().as_object_ref().unwrap().get_array_iter(){
                leaders.push(game_state.get_character(l.as_string_ref().unwrap().as_str()).clone());
            }
        }
        //append to this array the leader if its not already there, you would assume that the leader is the first element in the array, but not always
        let mut current = base.get("dynasty_head");
        if current.is_some(){
            if leaders.len() == 0{
                leaders.push(game_state.get_character(current.unwrap().as_string_ref().unwrap().as_str()).clone());
            }
        }
        else{
            current = base.get("head_of_house");
            if current.is_some(){
                if leaders.len() == 0{
                    leaders.push(game_state.get_character(current.unwrap().as_string_ref().unwrap().as_str()).clone());
                }
            }
        }
        let currency = base.get("prestige");
        let mut prestige_tot = 0.0;
        let mut prestige = 0.0;
        if currency.is_some(){
            let o = currency.unwrap().as_object_ref().unwrap();
            match o.get("accumulated").unwrap() {
                SaveFileValue::Object(ref o) => {
                    prestige_tot = o.as_ref().borrow().get_string_ref("value").parse::<f32>().unwrap();
                },
                SaveFileValue::String(ref o) => {
                    prestige_tot = o.as_ref().borrow().parse::<f32>().unwrap();
                },
            }
            match o.get("currency") {
                Some(v) => match v {
                    SaveFileValue::Object(ref o) => {
                        prestige = o.as_ref().borrow().get_string_ref("value").parse::<f32>().unwrap();
                    },
                    SaveFileValue::String(ref o) => {
                        prestige = o.as_ref().borrow().parse::<f32>().unwrap();
                    },
                },
                None => {}
            }
        }
        let parent_id = base.get("dynasty");
        let name:Option<Shared<String>> = match base.get("name") {
            Some(n) => Some(n.as_string()),
            None => None
        };
        Dynasty{
            name: name,
            parent: match parent_id {
                None => None,
                k => Some(game_state.get_dynasty(k.unwrap().as_string_ref().unwrap().as_str()).clone())
            },
            members: 0,
            houses: 0,
            prestige_tot: prestige_tot,
            prestige: prestige,
            perks: perks,
            leaders: leaders,
            id: base.get_name().parse::<u32>().unwrap()
        }
    }

    fn type_name() -> &'static str {
        "dynasty"
    }

    fn dummy() -> Self {
        Dynasty{
            name: Some(Shared::new("".to_owned().into())),
            parent: None,
            members: 0,
            houses: 0,
            prestige_tot: 0.0,
            prestige: 0.0,
            perks: Vec::new(),
            leaders: Vec::new(),
            id: 0
        }
    }

    fn init(&mut self, base:Ref<'_, GameObject>, game_state:&mut crate::game_state::GameState) {
        let mut perks = Vec::new();
        let perks_obj = base.get("perk");
        if perks_obj.is_some(){
            for p in perks_obj.unwrap().as_object_ref().unwrap().get_array_iter(){
                perks.push(p.as_string());
            }
        }
        let mut leaders = Vec::new();
        let leaders_obj = base.get("historical");
        if leaders_obj.is_some(){
            for l in leaders_obj.unwrap().as_object_ref().unwrap().get_array_iter(){
                leaders.push(game_state.get_character(l.as_string_ref().unwrap().as_str()).clone());
            }
        }
        let currency = base.get("prestige");
        let mut prestige_tot = 0.0;
        let mut prestige = 0.0;
        if currency.is_some(){
            let o = currency.unwrap().as_object_ref().unwrap();
            match o.get("accumulated").unwrap() {
                SaveFileValue::Object(ref o) => {
                    prestige_tot = o.as_ref().borrow().get_string_ref("value").parse::<f32>().unwrap();
                },
                SaveFileValue::String(ref o) => {
                    prestige_tot = o.as_ref().borrow().parse::<f32>().unwrap();
                },
            }
            match o.get("currency") {
                Some(v) => match v {
                    SaveFileValue::Object(ref o) => {
                        prestige = o.as_ref().borrow().get_string_ref("value").parse::<f32>().unwrap();
                    },
                    SaveFileValue::String(ref o) => {
                        prestige = o.as_ref().borrow().parse::<f32>().unwrap();
                    },
                },
                None => {}
            }
        }
        let parent_id = base.get("dynasty");
        let name:Option<Shared<String>> = match base.get("name") {
            Some(n) => Some(n.as_string()),
            None => None
        };
        self.name = name;
        self.parent = match parent_id {
            None => None,
            k => Some(game_state.get_dynasty(k.unwrap().as_string_ref().unwrap().as_str()).clone())
        };
        self.members = 0;
        self.houses = 0;
        self.prestige_tot = prestige_tot;
        self.prestige = prestige;
        self.perks = perks;
        self.leaders = leaders;
        self.id = base.get_name().parse::<u32>().unwrap();
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
        state.serialize_field("prestige_tot", &self.prestige_tot)?;
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
