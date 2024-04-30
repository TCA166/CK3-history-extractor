use std::cell::Ref;

use serde::Serialize;
use serde::ser::SerializeStruct;

use crate::game_object::GameObject;
use crate::game_state::GameState;

use super::{Character, Dynasty, GameObjectDerived, Shared};

pub struct LineageNode{
    pub character: Option<Shared<Character>>,
    pub date: Shared<String>,
    pub score: i32,
    pub prestige: i32,
    pub piety: i32,
    pub dread:f32,
    pub lifestyle: Shared<String>,
    pub perks:Vec<Shared<String>>,
    pub id: u32
}

fn get_perks(perks:&mut Vec<Shared<String>>, base:&Ref<'_, GameObject>){
    let perks_node = base.get("perks");
    if perks_node.is_some(){
        perks.push(perks_node.unwrap().as_string());
    }
}

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

fn get_lifestyle(base: &Ref<'_, GameObject>) -> Shared<String>{
    let lifestyle_node = base.get("lifestyle");
    if lifestyle_node.is_some() {
        lifestyle_node.unwrap().as_string()
    } else {
        Shared::new(String::new().into())
    }
}

impl GameObjectDerived for LineageNode{
    fn from_game_object(base:Ref<'_, GameObject>, game_state:&mut GameState) -> Self {
        let id = base.get_string_ref("character");
        let char = game_state.get_character(id.as_str());
        let mut perks: Vec<Shared<String>> = Vec::new();
        get_perks(&mut perks, &base);
        println!("{:?}", base);
        LineageNode { 
            character: Some(char),
            date: base.get("date").unwrap().as_string(),
            score: get_score(&base),
            prestige: get_prestige(&base),
            piety: get_piety(&base),
            dread: get_dread(&base),
            lifestyle: get_lifestyle(&base),
            perks: perks,
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
            lifestyle: Shared::new(String::new().into()),
            perks: Vec::new(),
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
        self.perks.clear();
        get_perks(&mut self.perks, &base);
    }

    fn get_id(&self) -> u32 {
        self.id
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
        state.serialize_field("perks", &self.perks)?;
        state.serialize_field("id", &self.id)?;
        state.end()
    }
}
