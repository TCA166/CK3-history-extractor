use std::cell::Ref;

use minijinja::{Environment, context};

use serde::Serialize;
use serde::ser::SerializeStruct;

use crate::game_object::GameObject;

use super::renderer::Renderable;

use super::{Culture, Dynasty, Faith, GameObjectDerived, Memory, Shared, Title};

pub struct Character {
    pub name: Shared<String>,
    pub nick: Option<Shared<String>>,
    pub birth: Shared<String>,
    pub dead: bool,
    pub date: Option<Shared<String>>,
    pub reason: Option<Shared<String>>,
    pub faith: Shared<Faith>,
    pub culture: Shared<Culture>,
    pub house: Option<Shared<Dynasty>>,
    pub skills: Vec<u8>,
    pub traits: Vec<Shared<String>>,
    pub recessive: Vec<Shared<String>>,
    pub spouses: Vec<Shared<Character>>,
    pub former: Vec<Shared<Character>>,
    pub children: Vec<Shared<Character>>,
    pub dna: Option<Shared<String>>,
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

    fn from_game_object(base:Ref<'_, GameObject>, game_state:&mut crate::game_state::GameState) -> Self {
        let keys = base.get_keys();
        let dead = keys.contains(&"date".to_string());
        let mut skills = Vec::new();
        for s in base.get_object_ref("skill").get_array_iter(){
            skills.push(s.as_string_ref().unwrap().parse::<u8>().unwrap());
        }
        let mut recessive = Vec::new();
        let rec_t = base.get("recessive_traits");
        if rec_t.is_some(){
            for r in rec_t.unwrap().as_object_ref().unwrap().get_array_iter(){
                recessive.push(r.as_string());
            }
        }
        let mut spouses = Vec::new();
        let mut former_spouses = Vec::new();
        let mut children = Vec::new();
        let family_data = base.get("family_data");
        if family_data.is_some(){
            let f = family_data.unwrap().as_object_ref().unwrap();
            let spouses_node = f.get("spouses");
            if spouses_node.is_some() {
                for s in spouses_node.unwrap().as_object_ref().unwrap().get_array_iter(){
                    spouses.push(game_state.get_character(s.as_string_ref().unwrap().as_str()).clone());
                }
            }
            let former_spouses_node = f.get("former_spouses");
            if former_spouses_node.is_some() {
                for s in former_spouses_node.unwrap().as_object_ref().unwrap().get_array_iter(){
                    former_spouses.push(game_state.get_character(s.as_string_ref().unwrap().as_str()).clone());
                }
            }
            let children_node = f.get("children");
            if children_node.is_some() {
                for s in children_node.unwrap().as_object_ref().unwrap().get_array_iter(){
                    children.push(game_state.get_character(s.as_string_ref().unwrap().as_str()).clone());
                }
            }
        }
        let dna = match base.get("dna"){
            Some(d) => Some(d.as_string()),
            None => None
        };
        let mut memories:Vec<Shared<Memory>> = Vec::new();
        let memory_node = base.get("memories");
        if memory_node.is_some(){
            for m in memory_node.unwrap().as_object_ref().unwrap().get_array_iter(){
                memories.push(game_state.get_memory(m.as_string_ref().unwrap().as_str()).clone());
            }
        }
        let mut traits = Vec::new();
        for t in base.get_object_ref("traits").get_array_iter(){
            traits.push(t.as_string());
        }
        let dynasty_id = base.get("dynasty_house");
        let mut piety = 0;
        let mut prestige = 0;
        let mut dread = 0;
        let mut strength = 0;
        let mut gold = 0;
        let mut kills: Vec<Shared<Character>> = Vec::new();
        let mut languages: Vec<Shared<String>> = Vec::new();
        if !dead {
            let alive_data = base.get("alive_data").unwrap().as_object_ref().unwrap();
            println!("{:?}", alive_data);
            piety = alive_data.get_object_ref("piety").get_object_ref("accumulated").get_string_ref("value").parse::<u32>().unwrap();
            prestige = alive_data.get_object_ref("prestige").get_object_ref("accumulated").get_string_ref("value").parse::<u32>().unwrap();
            let kills_node = alive_data.get("kills");
            if kills_node.is_some(){
                for k in kills_node.unwrap().as_object_ref().unwrap().get_array_iter(){
                    kills.push(game_state.get_character(k.as_string_ref().unwrap().as_str()).clone());
                }
            }
            for l in alive_data.get_object_ref("languages").get_array_iter(){
                languages.push(l.as_string());
            }
            let perk_node = alive_data.get("perks");
            if perk_node.is_some(){
                for p in perk_node.unwrap().as_object_ref().unwrap().get_array_iter(){
                    traits.push(p.as_string());
                }
            }
        }
        let mut titles: Vec<Shared<Title>> = Vec::new();
        let mut vassals:Vec<Shared<Character>> = Vec::new();
        let landed_data_node = base.get("landed_data");
        if landed_data_node.is_some(){
            let landed_data = landed_data_node.unwrap().as_object_ref().unwrap();
            dread = landed_data.get("dread").unwrap().as_string_ref().unwrap().parse::<u32>().unwrap();
            strength = landed_data.get("strength").unwrap().as_string_ref().unwrap().parse::<u32>().unwrap();
            gold = landed_data.get("gold").unwrap().as_string_ref().unwrap().parse::<u32>().unwrap();
            for t in landed_data.get("titles").unwrap().as_object_ref().unwrap().get_array_iter(){
                titles.push(game_state.get_title(t.as_string_ref().unwrap().as_str()).clone());
            }
            for v in landed_data.get("vassals").unwrap().as_object_ref().unwrap().get_array_iter(){
                vassals.push(game_state.get_character(v.as_string_ref().unwrap().as_str()).clone());
            }
        }
        Character{
            name: base.get("first_name").unwrap().as_string(),
            nick: base.get("nickname").map(|v| v.as_string()),
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
            house: match dynasty_id {
                Some(d) => Some(game_state.get_dynasty(d.as_string_ref().unwrap().as_str()).clone()),
                None => None
            },
            faith: game_state.get_faith(base.get("faith").unwrap().as_string_ref().unwrap().as_str()).clone(),
            culture: game_state.get_culture(base.get("culture").unwrap().as_string_ref().unwrap().as_str()).clone(),
            skills: skills,
            traits: traits,
            recessive:recessive,
            spouses: spouses,
            former: former_spouses,
            children: children,
            dna: dna,
            memories: memories,
            titles: titles,
            piety: piety,
            prestige: prestige,
            dread: dread,
            strength: strength,
            gold: gold,
            kills: kills,
            languages: languages,
            vassals: vassals
        }    
    }

    fn type_name() -> &'static str {
        "character"
    }

    fn dummy() -> Self {
        Character{
            name: Shared::new("".to_owned().into()),
            nick: None,
            birth: Shared::new("".to_owned().into()),
            dead: false,
            date: None,
            reason: None,
            faith: Shared::new(Faith::dummy().into()),
            culture: Shared::new(Culture::dummy().into()),
            house: None,
            skills: Vec::new(),
            traits: Vec::new(),
            recessive: Vec::new(),
            spouses: Vec::new(),
            former: Vec::new(),
            children: Vec::new(),
            dna: None,
            memories: Vec::new(),
            titles: Vec::new(),
            gold: 0,
            piety: 0,
            prestige: 0,
            dread: 0,
            strength: 0,
            kills: Vec::new(),
            languages: Vec::new(),
            vassals: Vec::new()
        }
    }

    fn init(&mut self, base:Ref<'_, GameObject>, game_state:&mut crate::game_state::GameState) {
        let keys = base.get_keys();
        self.dead = keys.contains(&"date".to_string());
        let mut skills = Vec::new();
        for s in base.get_object_ref("skill").get_array_iter(){
            skills.push(s.as_string_ref().unwrap().parse::<u8>().unwrap());
        }
        let mut recessive = Vec::new();
        let rec_t = base.get("recessive_traits");
        if rec_t.is_some(){
            for r in rec_t.unwrap().as_object_ref().unwrap().get_array_iter(){
                recessive.push(r.as_string());
            }
        }
        let mut spouses = Vec::new();
        let mut former_spouses = Vec::new();
        let mut children = Vec::new();
        let family_data = base.get("family_data");
        if family_data.is_some(){
            let f = family_data.unwrap().as_object_ref().unwrap();
            for s in f.get("spouses").unwrap().as_object_ref().unwrap().get_array_iter(){
                spouses.push(game_state.get_character(s.as_string_ref().unwrap().as_str()).clone());
            }
            for s in f.get("former_spouses").unwrap().as_object_ref().unwrap().get_array_iter(){
                former_spouses.push(game_state.get_character(s.as_string_ref().unwrap().as_str()).clone());
            }
            for s in f.get("children").unwrap().as_object_ref().unwrap().get_array_iter(){
                children.push(game_state.get_character(s.as_string_ref().unwrap().as_str()).clone());
            }
        }
        self.name = base.get("first_name").unwrap().as_string();
        self.nick = base.get("nickname").map(|v| v.as_string());
        self.birth = base.get("birth").unwrap().as_string();
        self.date = match self.dead {
            true => Some(base.get("date").unwrap().as_string()),
            false => None
        };
        self.reason = match self.dead {
            true => Some(base.get("reason").unwrap().as_string()),
            false => None
        };
        self.faith = game_state.get_faith(base.get("faith").unwrap().as_string_ref().unwrap().as_str()).clone();
        self.culture = game_state.get_culture(base.get("culture").unwrap().as_string_ref().unwrap().as_str()).clone();
        let dynasty_id = base.get("dynasty_house");
        self.house = match dynasty_id {
            Some(d) => Some(game_state.get_dynasty(d.as_string_ref().unwrap().as_str()).clone()),
            None => None
        };
        self.skills = skills;
        self.traits = base.get_object_ref("traits").get_array_iter().map(|t| t.as_string()).collect();
        self.recessive = recessive;
        self.spouses = spouses;
        self.former = former_spouses;
        self.children = children;
        let dna = match base.get("dna"){
            Some(d) => Some(d.as_string()),
            None => None
        };
        self.dna = dna;
        self.memories = base.get_object_ref("memories").get_array_iter().map(|m| game_state.get_memory(m.as_string_ref().unwrap().as_str()).clone()).collect();
        self.titles = base.get_object_ref("titles").get_array_iter().map(|t| game_state.get_title(t.as_string_ref().unwrap().as_str()).clone()).collect();
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
