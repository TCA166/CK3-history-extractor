use std::cell::Ref;

use minijinja::{Environment, context};

use serde::Serialize;
use serde::ser::SerializeStruct;

use crate::game_object::GameObject;

use super::renderer::Renderable;

use super::{Culture, Dynasty, Faith, GameObjectDerived, Memory, Shared, Title};

pub struct Character {
    pub name: Shared<String>,
    pub nick: Shared<String>,
    pub birth: Shared<String>,
    pub dead: bool,
    pub date: Option<Shared<String>>,
    pub reason: Option<Shared<String>>,
    pub faith: Shared<Faith>,
    pub culture: Shared<Culture>,
    pub house: Shared<Dynasty>,
    pub skills: Vec<u8>,
    pub traits: Vec<Shared<String>>,
    pub recessive: Vec<Shared<String>>,
    pub spouses: Vec<Shared<Character>>,
    pub former: Vec<Shared<Character>>,
    pub children: Vec<Shared<Character>>,
    pub dna: Shared<String>,
    pub memories: Vec<Shared<Memory>>,
    pub titles: Vec<Shared<Title>>,
    pub gold: u32,
    pub piety: u32,
    pub prestige: u32,
    pub dread: u32,
    pub strength: u32,
    pub kills: Vec<Shared<Character>>,
    pub languages: Vec<Shared<String>>,
    pub vassals: Vec<Shared<Character>>
}

impl GameObjectDerived for Character {

    fn from_game_object(base:Ref<'_, GameObject>, game_state:&crate::game_state::GameState) -> Self {
        let keys = base.get_keys();
        let dead = keys.contains(&"date".to_string());
        let mut skills = Vec::new();
        for s in base.get_object_ref("skills").get_array(){
            skills.push(s.as_string_ref().unwrap().parse::<u8>().unwrap());
        }
        Character{
            name: base.get("first_name").unwrap().as_string(),
            nick: base.get("nickname").unwrap().as_string(),
            birth: base.get("birth").unwrap().as_string(),
            dead: dead,
            date: match dead {
                true => Some(base.get("date").unwrap().as_string()),
                false => None
            },
            reason: match dead {
                true => Some(base.get("reason").unwrap().as_string()),
                false => None
            },
            faith: game_state.get_faith(base.get("religion").unwrap().as_string_ref().unwrap().as_str()).unwrap().clone(),
            culture: game_state.get_culture(base.get("faith").unwrap().as_string_ref().unwrap().as_str()).unwrap().clone(),
            house: game_state.get_dynasty(base.get("dynasty_house").unwrap().as_string_ref().unwrap().as_str()).unwrap().clone(),
            skills: skills,
            traits: base.get_object_ref("traits").get_array().iter().map(|t| t.as_string()).collect(),
            recessive: base.get_object_ref("recessive_traits").get_array().iter().map(|t| t.as_string()).collect(),
            spouses: base.get_object_ref("spouses").get_array().iter().map(|s| game_state.get_character(s.as_string_ref().unwrap().as_str()).unwrap().clone()).collect(),
            former: base.get_object_ref("former_spouses").get_array().iter().map(|s| game_state.get_character(s.as_string_ref().unwrap().as_str()).unwrap().clone()).collect(),
            children: base.get_object_ref("children").get_array().iter().map(|s| game_state.get_character(s.as_string_ref().unwrap().as_str()).unwrap().clone()).collect(),
            dna: base.get("dna").unwrap().as_string(),
            memories: base.get_object_ref("memories").get_array().iter().map(|m| game_state.get_memory(m.as_string_ref().unwrap().as_str()).unwrap().clone()).collect(),
            titles: base.get_object_ref("titles").get_array().iter().map(|t| game_state.get_title(t.as_string_ref().unwrap().as_str()).unwrap().clone()).collect(),
            gold: base.get("gold").unwrap().as_string_ref().unwrap().parse::<u32>().unwrap(),
            piety: base.get("piety").unwrap().as_string_ref().unwrap().parse::<u32>().unwrap(),
            prestige: base.get("prestige").unwrap().as_string_ref().unwrap().parse::<u32>().unwrap(),
            dread: base.get("dread").unwrap().as_string_ref().unwrap().parse::<u32>().unwrap(),
            strength: base.get("strength").unwrap().as_string_ref().unwrap().parse::<u32>().unwrap(),
            kills: base.get_object_ref("kills").get_array().iter().map(|k|game_state.get_character(k.as_string_ref().unwrap().as_str()).unwrap().clone()).collect(),
            languages: base.get_object_ref("languages").get_array().iter().map(|l| l.as_string()).collect(),
            vassals: base.get_object_ref("vassals").get_array().iter().map(|v| game_state.get_character(v.as_string_ref().unwrap().as_str()).unwrap().clone()).collect()
        }    
    }

    fn type_name() -> &'static str {
        "character"
    }
}

impl Serialize for Character {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Character", 27)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("nick", &self.nick)?;
        state.serialize_field("birth", &self.birth)?;
        state.serialize_field("dead", &self.dead)?;
        state.serialize_field("date", &self.date)?;
        state.serialize_field("reason", &self.reason)?;
        state.serialize_field("faith", &self.faith)?;
        state.serialize_field("culture", &self.culture)?;
        state.serialize_field("house", &self.house)?;
        state.serialize_field("skills", &self.skills)?;
        state.serialize_field("traits", &self.traits)?;
        state.serialize_field("recessive", &self.recessive)?;
        state.serialize_field("spouses", &self.spouses)?;
        state.serialize_field("former", &self.former)?;
        state.serialize_field("children", &self.children)?;
        state.serialize_field("dna", &self.dna)?;
        state.serialize_field("memories", &self.memories)?;
        state.serialize_field("titles", &self.titles)?;
        state.serialize_field("gold", &self.gold)?;
        state.serialize_field("piety", &self.piety)?;
        state.serialize_field("prestige", &self.prestige)?;
        state.serialize_field("dread", &self.dread)?;
        state.serialize_field("strength", &self.strength)?;
        state.serialize_field("kills", &self.kills)?;
        state.serialize_field("languages", &self.languages)?;
        state.serialize_field("vassals", &self.vassals)?;
        state.end()
    }
}

impl Renderable for Character {
    fn render(&self, env: &Environment, template_name: &'static String) -> String {
        let ctx = context! {character=>self};
        env.get_template(template_name).unwrap().render(&ctx).unwrap()
    }
}
