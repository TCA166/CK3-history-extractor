use minijinja::{Environment, context};

use serde::Serialize;
use serde::ser::SerializeStruct;

use crate::game_object::GameObject;

use super::renderer::Renderable;

use super::{Faith, Culture, Dynasty, Memory, Title, GameObjectDerived};

use std::rc::Rc;

pub struct Character {
    pub name: Rc<String>,
    pub nick: Rc<String>,
    pub birth: Rc<String>,
    pub dead: bool,
    pub date: Option<Rc<String>>,
    pub reason: Option<Rc<String>>,
    pub faith: Rc<Faith>,
    pub culture: Rc<Culture>,
    pub house: Rc<Dynasty>,
    pub skills: Vec<u8>,
    pub traits: Vec<Rc<String>>,
    pub recessive: Vec<Rc<String>>,
    pub spouses: Vec<Rc<Character>>,
    pub former: Vec<Rc<Character>>,
    pub children: Vec<Rc<Character>>,
    pub dna: Rc<String>,
    pub memories: Vec<Rc<Memory>>,
    pub titles: Vec<Rc<Title>>,
    pub gold: u32,
    pub piety: u32,
    pub prestige: u32,
    pub dread: u32,
    pub strength: u32,
    pub kills: Vec<Rc<Character>>,
    pub languages: Vec<Rc<String>>,
    pub vassals: Vec<Rc<Character>>
}

impl GameObjectDerived for Character {

    fn from_game_object(base:&'_ GameObject, game_state:&crate::game_state::GameState) -> Self {
        let keys = base.get_keys();
        let dead = keys.contains(&"date".to_string());
        let mut skills = Vec::new();
        for s in base.get("skills").unwrap().as_array().unwrap(){
            skills.push(s.as_string().unwrap().parse::<u8>().unwrap());
        }
        Character{
            name: base.get("first_name").unwrap().as_string().unwrap(),
            nick: base.get("nickname").unwrap().as_string().unwrap(),
            birth: base.get("birth").unwrap().as_string().unwrap(),
            dead: dead,
            date: match dead {
                true => Some(base.get("date").unwrap().as_string().unwrap()),
                false => None
            },
            reason: match dead {
                true => Some(base.get("reason").unwrap().as_string().unwrap()),
                false => None
            },
            faith: Rc::from(game_state.get_faith(base.get("religion").unwrap().as_string().unwrap().as_str()).unwrap().clone()),
            culture: Rc::from(game_state.get_culture(base.get("faith").unwrap().as_string().unwrap().as_str()).unwrap().clone()),
            house: Rc::from(game_state.get_dynasty(base.get("dynasty_house").unwrap().as_string().unwrap().as_str()).unwrap().clone()),
            skills: skills,
            traits: base.get("traits").unwrap().as_array().unwrap().iter().map(|t| Rc::from(t.as_string().unwrap())).collect(),
            recessive: base.get("recessive_traits").unwrap().as_array().unwrap().iter().map(|t| Rc::from(t.as_string().unwrap())).collect(),
            spouses: base.get("spouses").unwrap().as_array().unwrap().iter().map(|s| Rc::from(game_state.get_character(s.as_string().unwrap().as_str()).unwrap().clone())).collect(),
            former: base.get("former_spouses").unwrap().as_array().unwrap().iter().map(|s| Rc::from(game_state.get_character(s.as_string().unwrap().as_str()).unwrap().clone())).collect(),
            children: base.get("children").unwrap().as_array().unwrap().iter().map(|s| Rc::from(game_state.get_character(s.as_string().unwrap().as_str()).unwrap().clone())).collect(),
            dna: Rc::from(base.get("dna").unwrap().as_string().unwrap()),
            memories: base.get("memories").unwrap().as_array().unwrap().iter().map(|m| Rc::from(game_state.get_memory(m.as_string().unwrap().as_str()).unwrap().clone())).collect(),
            titles: base.get("titles").unwrap().as_array().unwrap().iter().map(|t| Rc::from(game_state.get_title(t.as_string().unwrap().as_str()).unwrap().clone())).collect(),
            gold: base.get("gold").unwrap().as_string().unwrap().parse::<u32>().unwrap(),
            piety: base.get("piety").unwrap().as_string().unwrap().parse::<u32>().unwrap(),
            prestige: base.get("prestige").unwrap().as_string().unwrap().parse::<u32>().unwrap(),
            dread: base.get("dread").unwrap().as_string().unwrap().parse::<u32>().unwrap(),
            strength: base.get("strength").unwrap().as_string().unwrap().parse::<u32>().unwrap(),
            kills: base.get("kills").unwrap().as_array().unwrap().iter().map(|k| Rc::from(game_state.get_character(k.as_string().unwrap().as_str()).unwrap().clone())).collect(),
            languages: base.get("languages").unwrap().as_array().unwrap().iter().map(|l| Rc::from(l.as_string().unwrap())).collect(),
            vassals: base.get("vassals").unwrap().as_array().unwrap().iter().map(|v| Rc::from(game_state.get_character(v.as_string().unwrap().as_str()).unwrap().clone())).collect()
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
