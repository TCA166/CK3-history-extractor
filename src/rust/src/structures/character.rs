use minijinja::{Environment, context};

use serde::Serialize;
use serde::ser::SerializeStruct;

use super::renderer::Renderable;

use super::Faith;
use super::Culture;
use super::Dynasty;
use super::Memory;
use super::Title;

pub struct Character<'a> {
    name: &'a String,
    nick: &'a String,
    birth: &'a String,
    dead: &'a String,
    date: Option<&'a String>,
    reason: Option<&'a String>,
    faith: &'a Faith<'a>,
    culture: &'a Culture<'a>,
    house: &'a Dynasty<'a>,
    skills: Vec<u8>,
    traits: Vec<&'a String>,
    recessive: Vec<&'a String>,
    spouses: Vec<&'a Character<'a>>,
    former: Vec<&'a Character<'a>>,
    children: Vec<&'a Character<'a>>,
    dna: &'a String,
    memories: Vec<&'a Memory<'a>>,
    titles: Vec<&'a Title<'a>>,
    gold: u32,
    piety: u32,
    prestige: u32,
    dread: u32,
    strength: u32,
    kills: Vec<&'a Character<'a>>,
    languages: Vec<&'a String>,
    vassals: Vec<&'a Character<'a>>
}

impl Serialize for Character<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Character", 27)?;
        state.serialize_field("name", self.name)?;
        state.serialize_field("nick", self.nick)?;
        state.serialize_field("birth", self.birth)?;
        state.serialize_field("dead", self.dead)?;
        state.serialize_field("date", &self.date)?;
        state.serialize_field("reason", &self.reason)?;
        state.serialize_field("faith", self.faith)?;
        state.serialize_field("culture", self.culture)?;
        state.serialize_field("house", self.house)?;
        state.serialize_field("skills", &self.skills)?;
        state.serialize_field("traits", &self.traits)?;
        state.serialize_field("recessive", &self.recessive)?;
        state.serialize_field("spouses", &self.spouses)?;
        state.serialize_field("former", &self.former)?;
        state.serialize_field("children", &self.children)?;
        state.serialize_field("dna", self.dna)?;
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

impl Renderable for Character<'_>{
    fn render(&self, env: &Environment, template_name: &'static String) -> String {
        let ctx = context! {character=>self};
        env.get_template(template_name).unwrap().render(&ctx).unwrap()    
    }
}
