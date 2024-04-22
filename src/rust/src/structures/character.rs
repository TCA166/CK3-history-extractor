use minijinja::{Environment, context};

use serde::Serialize;
use serde::ser::SerializeStruct;

use crate::game_object::GameObject;

use super::renderer::Renderable;

use super::{Faith, Culture, Dynasty, Memory, Title, GameObjectDerived};

use std::rc::Rc;

pub struct Character {
    name: Rc<String>,
    nick: Rc<String>,
    birth: Rc<String>,
    dead: Rc<String>,
    date: Option<Rc<String>>,
    reason: Option<Rc<String>>,
    faith: Rc<Faith>,
    culture: Rc<Culture>,
    house: Rc<Dynasty>,
    skills: Vec<u8>,
    traits: Vec<Rc<String>>,
    recessive: Vec<Rc<String>>,
    spouses: Vec<Rc<Character>>,
    former: Vec<Rc<Character>>,
    children: Vec<Rc<Character>>,
    dna: Rc<String>,
    memories: Vec<Rc<Memory>>,
    titles: Vec<Rc<Title>>,
    gold: u32,
    piety: u32,
    prestige: u32,
    dread: u32,
    strength: u32,
    kills: Vec<Rc<Character>>,
    languages: Vec<Rc<String>>,
    vassals: Vec<Rc<Character>>
}

impl GameObjectDerived for Character {
    fn from_game_object(base:&'_ GameObject, game_state:&std::collections::HashMap<String, std::collections::HashMap<String, super::GameObjectDerivedType>>) -> Self {
        Character {
            
        }
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
