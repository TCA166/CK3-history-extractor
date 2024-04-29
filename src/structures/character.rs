use std::cell::Ref;

use minijinja::{Environment, context};

use serde::Serialize;
use serde::ser::SerializeStruct;

use crate::game_object::GameObject;
use crate::game_state::GameState;

use super::renderer::Renderable;

use super::{Culture, Dynasty, Faith, GameObjectDerived, Memory, Shared, Title};

pub struct Character {
    pub id: u32,
    pub name: Shared<String>,
    pub nick: Option<Shared<String>>,
    pub birth: Shared<String>,
    pub dead: bool,
    pub date: Option<Shared<String>>,
    pub reason: Option<Shared<String>>,
    pub faith: Shared<Faith>,
    pub culture: Shared<Culture>,
    pub house: Option<Shared<Dynasty>>,
    pub skills: Vec<i8>,
    pub traits: Vec<Shared<String>>,
    pub recessive: Vec<Shared<String>>,
    pub spouses: Vec<Shared<Character>>,
    pub former: Vec<Shared<Character>>,
    pub children: Vec<Shared<Character>>,
    pub dna: Option<Shared<String>>,
    pub memories: Vec<Shared<Memory>>,
    pub titles: Vec<Shared<Title>>,
    pub gold: f32,
    pub piety: f32,
    pub prestige: f32,
    pub dread: f32,
    pub strength: f32,
    pub kills: Vec<Shared<Character>>,
    pub languages: Vec<Shared<String>>,
    pub vassals: Vec<Shared<Character>>
}

//TODO seperate each getting segment into a separate function

fn get_faith(house:&Option<Shared<Dynasty>>, base:&GameObject, game_state:&mut GameState) -> Shared<Faith>{
    let faith_node = base.get("faith");
    if faith_node.is_some(){
        return game_state.get_faith(faith_node.unwrap().as_string_ref().unwrap().as_str()).clone();
    }
    else{
        let h = house.as_ref().unwrap().borrow();
        if h.leaders.len() == 0 {
            if h.parent.is_some(){
                let p = h.parent.as_ref().unwrap().borrow();
                if p.leaders.len() != 0 {
                    return p.leaders[0].borrow().faith.clone();
                }
                else{
                    panic!("No faith found");
                }
            }
            //if we made it here it means we need to search for faith in culture
            panic!("No faith found");
        }
        else{
            let mut i = 0;
            loop {
                let l = h.leaders[i].try_borrow();
                if l.is_ok(){
                    return l.unwrap().faith.clone();
                }
                i += 1;
            }
        }
    }
}

fn get_culture(house:&Option<Shared<Dynasty>>, base:&GameObject, game_state:&mut GameState) -> Shared<Culture>{
    let culture_node = base.get("culture");
    if culture_node.is_some(){
        return game_state.get_culture(culture_node.unwrap().as_string_ref().unwrap().as_str()).clone();
    }
    else{
        let h = house.as_ref().unwrap().borrow();
        if h.leaders.len() == 0 {
            let p = h.parent.as_ref().unwrap().borrow();
            return p.leaders[0].borrow().culture.clone();
        }
        else{
            return h.leaders[0].borrow().culture.clone();
        }
    }
}

fn get_skills(skills:&mut Vec<i8>, base:&GameObject){
    for s in base.get_object_ref("skill").get_array_iter(){
        skills.push(s.as_string_ref().unwrap().parse::<i8>().unwrap());
    }
}

fn is_dead(base:&GameObject) -> bool{
    let keys = base.get_keys();
    keys.contains(&"date".to_string())
}

fn get_recessive(recessive:&mut Vec<Shared<String>>, base:&GameObject){
    let rec_t = base.get("recessive_traits");
    if rec_t.is_some(){
        for r in rec_t.unwrap().as_object_ref().unwrap().get_array_iter(){
            recessive.push(r.as_string());
        }
    }
}

fn get_family(spouses:&mut Vec<Shared<Character>>, former_spouses:&mut Vec<Shared<Character>>, children:&mut Vec<Shared<Character>>, base:&GameObject, game_state:&mut GameState){
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
}

fn get_memories(memories:&mut Vec<Shared<Memory>>, base:&GameObject, game_state:&mut GameState){
    let memory_node = base.get("memories");
    if memory_node.is_some(){
        for m in memory_node.unwrap().as_object_ref().unwrap().get_array_iter(){
            memories.push(game_state.get_memory(m.as_string_ref().unwrap().as_str()).clone());
        }
    }
}

fn get_traits(traits:&mut Vec<Shared<String>>, base:&GameObject){
    let traits_node = base.get("traits");
    if traits_node.is_some(){
        for t in traits_node.unwrap().as_object_ref().unwrap().get_array_iter(){
            traits.push(t.as_string());
        }
    }
}

fn parse_alive_data(base:&GameObject, piety:&mut f32, prestige:&mut f32, kills:&mut Vec<Shared<Character>>, languages:&mut Vec<Shared<String>>, traits:&mut Vec<Shared<String>>, game_state:&mut GameState){
    let alive_node = base.get("alive_data");
    if alive_node.is_some(){
        let alive_data = base.get("alive_data").unwrap().as_object_ref().unwrap();
        *piety = alive_data.get_object_ref("piety").get_string_ref("accumulated").parse::<f32>().unwrap();
        *prestige = alive_data.get_object_ref("prestige").get_string_ref("accumulated").parse::<f32>().unwrap();
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
}

fn get_landed_data(gold:&mut f32, dread:&mut f32, strength:&mut f32, titles:&mut Vec<Shared<Title>>, vassals:&mut Vec<Shared<Character>>, base:&GameObject, game_state:&mut GameState){
    let landed_data_node = base.get("landed_data");
    if landed_data_node.is_some(){
        let landed_data = landed_data_node.unwrap().as_object_ref().unwrap();
        let dread_node = landed_data.get("dread");
        if dread_node.is_some(){
            *dread = dread_node.unwrap().as_string_ref().unwrap().parse::<f32>().unwrap();
        }
        let strength_node = landed_data.get("strength");
        if strength_node.is_some(){
            *strength = landed_data.get("strength").unwrap().as_string_ref().unwrap().parse::<f32>().unwrap();
        }
        let gold_node = landed_data.get("gold");
        if gold_node.is_some(){
            *gold = gold_node.unwrap().as_string_ref().unwrap().parse::<f32>().unwrap();
        }
        let titles_node = landed_data.get("titles");
        if titles_node.is_some(){
            for t in titles_node.unwrap().as_object_ref().unwrap().get_array_iter(){
                titles.push(game_state.get_title(t.as_string_ref().unwrap().as_str()).clone());
            }
        }
        let vassals_node = landed_data.get("vassals");
        if vassals_node.is_some(){
            for v in landed_data.get("vassals").unwrap().as_object_ref().unwrap().get_array_iter(){
                vassals.push(game_state.get_character(v.as_string_ref().unwrap().as_str()).clone());
            }
        }
    }
}

fn get_dynasty(base:&GameObject, game_state:&mut GameState) -> Option<Shared<Dynasty>>{
    let dynasty_id = base.get("dynasty_house");
    if dynasty_id.is_some(){
        return Some(game_state.get_dynasty(dynasty_id.unwrap().as_string_ref().unwrap().as_str()).clone());
    }
    None
}

impl GameObjectDerived for Character {

    fn from_game_object(base:Ref<'_, GameObject>, game_state:&mut GameState) -> Self {
        let dead = is_dead(&base);
        //find skills
        let mut skills = Vec::new();
        get_skills(&mut skills, &base);
        //find recessive traits
        let mut recessive = Vec::new();
        get_recessive(&mut recessive, &base);
        //find family data
        let mut spouses = Vec::new();
        let mut former_spouses = Vec::new();
        let mut children = Vec::new();
        get_family(&mut spouses, &mut former_spouses, &mut children, &base, game_state);
        //find dna
        let dna = match base.get("dna"){
            Some(d) => Some(d.as_string()),
            None => None
        };
        //find memories
        let mut memories:Vec<Shared<Memory>> = Vec::new();
        get_memories(&mut memories, &base, game_state);
        //find traits
        let mut traits = Vec::new();
        get_traits(&mut traits, &base);
        //find alive data
        let mut piety = 0.0;
        let mut prestige = 0.0;
        let mut kills: Vec<Shared<Character>> = Vec::new();
        let mut languages: Vec<Shared<String>> = Vec::new();
        if !dead {
            parse_alive_data(&base, &mut piety, &mut prestige, &mut kills, &mut languages, &mut traits, game_state);
        }
        //find landed data
        let mut gold = 0.0;
        let mut dread = 0.0;
        let mut strength = 0.0;
        let mut titles: Vec<Shared<Title>> = Vec::new();
        let mut vassals:Vec<Shared<Character>> = Vec::new();
        get_landed_data(&mut gold, &mut dread, &mut strength, &mut titles, &mut vassals, &base, game_state);
        //find house
        let house = get_dynasty(&base, game_state);
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
            house: house.clone(),
            faith: get_faith(&house, &base, game_state),
            culture: get_culture(&house, &base, game_state),
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
            vassals: vassals,
            id: base.get_name().parse::<u32>().unwrap()
        }    
    }

    fn dummy(id:u32) -> Self {
        Character{
            name: Shared::new("".to_owned().into()),
            nick: None,
            birth: Shared::new("".to_owned().into()),
            dead: false,
            date: None,
            reason: None,
            faith: Shared::new(Faith::dummy(0).into()),
            culture: Shared::new(Culture::dummy(0).into()),
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
            gold: 0.0,
            piety: 0.0,
            prestige: 0.0,
            dread: 0.0,
            strength: 0.0,
            kills: Vec::new(),
            languages: Vec::new(),
            vassals: Vec::new(),
            id: id
        }
    }

    fn init(&mut self, base:Ref<'_, GameObject>, game_state:&mut GameState) {
        let dead = is_dead(&base);
        //find skills
        get_skills(&mut self.skills, &base);
        //find recessive traits
        get_recessive(&mut self.recessive, &base);
        get_family(&mut self.spouses, &mut self.former, &mut self.children, &base, game_state);
        //find dna
        let dna = match base.get("dna"){
            Some(d) => Some(d.as_string()),
            None => None
        };
        //find memories
        get_memories(&mut self.memories, &base, game_state);
        //find traits
        get_traits(&mut self.traits, &base);
        //find alive data
        if !dead {
            parse_alive_data(&base, &mut self.piety, &mut self.prestige, &mut self.kills, &mut self.languages, &mut self.traits, game_state);
        }
        //find landed data
        get_landed_data(&mut self.gold, &mut self.dread, &mut self.strength, &mut self.titles, &mut self.vassals, &base, game_state);
        //find house
        let house = get_dynasty(&base, game_state);
        self.name.clone_from(&base.get("first_name").unwrap().as_string());
        self.nick = base.get("nickname").map(|v| v.as_string());
        self.birth.clone_from(&base.get("birth").unwrap().as_string());
        self.dead = dead;
        self.date = match dead {
            true => Some(base.get("date").unwrap().as_string()),
            false => None
        };
        self.reason = match dead {
            true => Some(base.get("reason").unwrap().as_string()),
            false => None
        };
        self.house = house.clone();
        self.faith.clone_from(&get_faith(&house, &base, game_state));
        self.culture.clone_from(&get_culture(&house, &base, game_state));
        self.dna = dna;
    }

    fn get_id(&self) -> u32 {
        self.id
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
