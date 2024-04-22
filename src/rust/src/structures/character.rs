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
    pub dead: Rc<String>,
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
