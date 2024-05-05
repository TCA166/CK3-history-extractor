use std::cell::Ref;

use serde::Serialize;
use serde::ser::SerializeStruct;

use crate::game_object::GameObject;
use crate::game_state::GameState;

use super::{Character, GameObjectDerived, Shared};

/// A struct representing a lineage node in the game
pub struct LineageNode{
    character: Option<Shared<Character>>,
    date: Shared<String>,
    score: i32,
    prestige: i32,
    piety: i32,
    dread:f32,
    lifestyle: Option<Shared<String>>,
    perk:Option<Shared<String>>, //in older version this was a list, guess it no longer is
    id: u32
}

impl LineageNode {
    /// Gets the character associated with the lineage node
    pub fn get_character(&self) -> Shared<Character> {
        self.character.as_ref().unwrap().clone()
    }
}

///Gets the perk of the lineage node
fn get_perk(base:&Ref<'_, GameObject>) -> Option<Shared<String>>{
    let perks_node = base.get("perk");
    if perks_node.is_some(){
        Some(perks_node.unwrap().as_string())
    }
    else{
        None
    }
}

///Gets the dread of the lineage node
fn get_dread(base:&Ref<'_, GameObject>) -> f32{
    let dread;
    let dread_node = base.get("dread");
    if dread_node.is_some() {
        dread = dread_node.unwrap().as_string_ref().unwrap().parse::<f32>().unwrap();
    }
    else{
        dread = 0.0;
    }
    return dread;
}

///Gets the score of the lineage node
fn get_score(base: &Ref<'_, GameObject>) -> i32 {
    let score;
    let score_node = base.get("score");
    if score_node.is_some() {
        score = score_node.unwrap().as_string_ref().unwrap().parse::<i32>().unwrap();
    } else {
        score = 0;
    }
    score
}

///Gets the prestige of the lineage node
fn get_prestige(base: &Ref<'_, GameObject>) -> i32 {
    let prestige;
    let prestige_node = base.get("prestige");
    if prestige_node.is_some() {
        prestige = prestige_node.unwrap().as_string_ref().unwrap().parse::<i32>().unwrap();
    } else {
        prestige = 0;
    }
    prestige
}

///Gets the piety of the lineage node
fn get_piety(base: &Ref<'_, GameObject>) -> i32 {
    let piety;
    let piety_node = base.get("piety");
    if piety_node.is_some() {
        piety = piety_node.unwrap().as_string_ref().unwrap().parse::<i32>().unwrap();
    }
    else{
        piety = 0;
    }
    piety
}

///Gets the lifestyle of the lineage node
fn get_lifestyle(base: &Ref<'_, GameObject>) -> Option<Shared<String>>{
    let lifestyle_node = base.get("lifestyle");
    if lifestyle_node.is_some() {
        Some(lifestyle_node.unwrap().as_string())
    } else {
        None
    }
}

impl GameObjectDerived for LineageNode{
    fn from_game_object(base:Ref<'_, GameObject>, game_state:&mut GameState) -> Self {
        let id = base.get_string_ref("character");
        let char = game_state.get_character(id.as_str());
        LineageNode { 
            character: Some(char),
            date: base.get("date").unwrap().as_string(),
            score: get_score(&base),
            prestige: get_prestige(&base),
            piety: get_piety(&base),
            dread: get_dread(&base),
            lifestyle: get_lifestyle(&base),
            perk: get_perk(&base),
            id: id.parse::<u32>().unwrap()
        }
    }

    fn dummy(id:u32) -> Self {
        LineageNode{
            character: None,
            date: Shared::new(String::new().into()),
            score: 0,
            prestige: 0,
            piety: 0,
            dread: 0.0,
            lifestyle: None,
            perk: None,
            id: id
        }
    }

    fn init(&mut self, base:Ref<'_, GameObject>, game_state:&mut GameState) {
        let character_id = base.get_string_ref("character");
        self.character = Some(game_state.get_character(character_id.as_str()));
        self.score = get_score(&base);
        self.prestige = get_prestige(&base);
        self.piety = get_piety(&base);
        self.dread = get_dread(&base);
        self.lifestyle = get_lifestyle(&base);
        self.perk = get_perk(&base);
    }

    fn get_id(&self) -> u32 {
        self.id
    }

    fn get_name(&self) -> Shared<String> {
        self.character.as_ref().unwrap().borrow().get_name()
    }
}

impl Serialize for LineageNode{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("LineageNode", 11)?;
        state.serialize_field("character", &self.character)?;
        state.serialize_field("date", &self.date)?;
        state.serialize_field("score", &self.score)?;
        state.serialize_field("prestige", &self.prestige)?;
        state.serialize_field("piety", &self.piety)?;
        state.serialize_field("dread", &self.dread)?;
        state.serialize_field("lifestyle", &self.lifestyle)?;
        state.serialize_field("perk", &self.perk)?;
        state.serialize_field("id", &self.id)?;
        state.end()
    }
}
