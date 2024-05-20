use std::slice::Iter;

use minijinja::context;

use serde::{Serialize, ser::SerializeStruct};

use crate::{game_object::{GameObject, GameString, SaveFileValue}, game_state::GameState, graph::Grapher, localizer::Localizer, map::GameMap, types::{Wrapper, WrapperMut}};

use crate::renderer::{Cullable, Renderable, Renderer};

use super::{serialize_array, Culture, DerivedRef, DummyInit, Dynasty, Faith, GameId, GameObjectDerived, Memory, Shared, Title};

/// Represents a character in the game.
/// Implements [GameObjectDerived], [Renderable] and [Cullable].
pub struct Character {
    id: GameId,
    /// The name of the character, you can assume this is always present.
    name: Option<GameString>,
    nick: Option<GameString>,
    birth: Option<GameString>,
    dead: bool,
    date: Option<GameString>,
    reason: Option<GameString>,
    faith: Option<Shared<Faith>>,
    culture: Option<Shared<Culture>>,
    house: Option<Shared<Dynasty>>,
    skills: Vec<i8>,
    traits: Vec<GameString>,
    recessive: Vec<GameString>,
    spouses: Vec<Shared<Character>>,
    former: Vec<Shared<Character>>,
    children: Vec<Shared<Character>>,
    parents: Vec<Shared<Character>>,
    dna: Option<GameString>,
    memories: Vec<Shared<Memory>>,
    titles: Vec<Shared<Title>>,
    gold: f32,
    piety: f32,
    prestige: f32,
    dread: f32,
    strength: f32,
    kills: Vec<Shared<Character>>,
    languages: Vec<GameString>,
    vassals: Vec<Shared<DerivedRef<Character>>>,
    female:bool,
    depth: usize,
    localized: bool,
    name_localized: bool
}

// So both faith and culture can be stored for a character in the latest leader of their house. 
// The problem with reading that now is that while Houses are already likely loaded,
// the characters that the houses hold reference to are likely still dummy, so we can't read the faith and culture from the house leader.
// So we will be returning None for now in case either is missing, but later during serialization read the one from house.

/// Gets the faith of the character
fn get_faith(base:&GameObject, game_state:&mut GameState) -> Option<Shared<Faith>>{
    let faith_node = base.get("faith");
    if faith_node.is_some(){
        return Some(game_state.get_faith(&faith_node.unwrap().as_id()).clone());
    }
    None
}

/// Gets the culture of the character
fn get_culture(base:&GameObject, game_state:&mut GameState) -> Option<Shared<Culture>>{
    let culture_node = base.get("culture");
    if culture_node.is_some(){
        return Some(game_state.get_culture(&culture_node.unwrap().as_id()).clone())
    }
    None
}

/// Gets the skills of the character
fn get_skills(skills:&mut Vec<i8>, base:&GameObject){
    for s in base.get_object_ref("skill").get_array_iter(){
        skills.push(s.as_string().parse::<i8>().unwrap());
    }
}

/// Parses the dead_data field of the character
fn get_dead(dead:&mut bool, reason:&mut Option<GameString>, data:&mut Option<GameString>, titles:&mut Vec<Shared<Title>>, base:&GameObject, game_state:&mut GameState){
    let dead_data = base.get("dead_data");
    if dead_data.is_some(){
        *dead = true;
        let o = dead_data.unwrap().as_object().unwrap();
        let reason_node = o.get("reason");
        if reason_node.is_some(){
            *reason = Some(reason_node.unwrap().as_string());
        }
        let domain_node = o.get("domain");
        if domain_node.is_some(){
            for t in domain_node.unwrap().as_object().unwrap().get_array_iter(){
                titles.push(game_state.get_title(&t.as_id()));
            }
        }
        *data = Some(o.get("date").unwrap().as_string());
    }
}

/// Gets the recessive traits of the character
fn get_recessive(recessive:&mut Vec<GameString>, base:&GameObject){
    let rec_t = base.get("recessive_traits");
    if rec_t.is_some(){
        for r in rec_t.unwrap().as_object().unwrap().get_array_iter(){
            recessive.push(r.as_string());
        }
    }
}

/// Parses the family_data field of the character
fn get_family(self_id:GameId, spouses:&mut Vec<Shared<Character>>, former_spouses:&mut Vec<Shared<Character>>, children:&mut Vec<Shared<Character>>, base:&GameObject, game_state:&mut GameState){
    let family_data = base.get("family_data");
    if family_data.is_some(){
        let f = family_data.unwrap().as_object().unwrap();
        let former_spouses_node = f.get("former_spouses");
        if former_spouses_node.is_some() {
            for s in former_spouses_node.unwrap().as_object().unwrap().get_array_iter(){
                former_spouses.push(game_state.get_character(&s.as_id()).clone());
            }
        }
        let spouse_node = f.get("spouse");
        if spouse_node.is_some() {
            match spouse_node.unwrap() {
                SaveFileValue::Object(o) => {
                    for s in o.get_array_iter(){
                        let c = game_state.get_character(&s.as_id()).clone();
                        let contains = former_spouses.iter().any(|x| x.get_internal().get_id() == c.get_internal().get_id());
                        if !contains{
                            spouses.push(c);
                        }
                    }
                }
                SaveFileValue::String(o) => {
                    let c = game_state.get_character(&o.parse::<GameId>().unwrap()).clone();
                    let contains = former_spouses.iter().any(|x| x.get_internal().get_id() == c.get_internal().get_id());
                    if !contains{
                        spouses.push(c);
                    }
                }
            }
        }
        let primary_spouse_node = f.get("primary_spouse");
        if primary_spouse_node.is_some() {
            let c = game_state.get_character(&primary_spouse_node.unwrap().as_id()).clone();
            let mut contains = former_spouses.iter().any(|x| x.get_internal().get_id() == c.get_internal().get_id());
            contains = contains || spouses.iter().any(|x| x.get_internal().get_id() == c.get_internal().get_id());
            if !contains{
                spouses.push(c);
            }
        }
        let children_node = f.get("child");
        if children_node.is_some() {
            let parent = game_state.get_character(&self_id);
            for s in children_node.unwrap().as_object().unwrap().get_array_iter(){
                let c = game_state.get_character(&s.as_id()).clone();
                c.get_internal_mut().register_parent(parent.clone());
                children.push(c);
            }
        }
    }
}

/// Gets the traits of the character
fn get_traits(traits:&mut Vec<GameString>, base:&GameObject, game_state:&mut GameState){
    let traits_node = base.get("traits");
    if traits_node.is_some(){
        for t in traits_node.unwrap().as_object().unwrap().get_array_iter(){
            let index = t.as_string().parse::<u16>().unwrap();
            traits.push(game_state.get_trait(index));
        }
    }
}

/// Parses the alive_data field of the character
fn parse_alive_data(base:&GameObject, piety:&mut f32, prestige:&mut f32, gold:&mut f32, kills:&mut Vec<Shared<Character>>, languages:&mut Vec<GameString>, traits:&mut Vec<GameString>, memories:&mut Vec<Shared<Memory>>, game_state:&mut GameState){
    let alive_node = base.get("alive_data");
    if alive_node.is_some(){
        let alive_data = base.get("alive_data").unwrap().as_object().unwrap();
        *piety = alive_data.get_object_ref("piety").get_string_ref("accumulated").parse::<f32>().unwrap();
        *prestige = alive_data.get_object_ref("prestige").get_string_ref("accumulated").parse::<f32>().unwrap();
        let kills_node = alive_data.get("kills");
        if kills_node.is_some(){
            for k in kills_node.unwrap().as_object().unwrap().get_array_iter(){
                kills.push(game_state.get_character(&k.as_id()).clone());
            }
        }
        let gold_node = alive_data.get("gold");
        if gold_node.is_some(){
            *gold = gold_node.unwrap().as_string().parse::<f32>().unwrap();
        }
        for l in alive_data.get_object_ref("languages").get_array_iter(){
            languages.push(l.as_string());
        }
        let perk_node = alive_data.get("perks");
        if perk_node.is_some(){
            for p in perk_node.unwrap().as_object().unwrap().get_array_iter(){
                traits.push(p.as_string());
            }
        }
        let memory_node = alive_data.get("memories");
        if memory_node.is_some(){
            for m in memory_node.unwrap().as_object().unwrap().get_array_iter(){
                memories.push(game_state.get_memory(&m.as_id()).clone());
            }
        }
    }
}

/// Parses the landed_data field of the character
fn get_landed_data(dread:&mut f32, strength:&mut f32, titles:&mut Vec<Shared<Title>>, vassals:&mut Vec<Shared<DerivedRef<Character>>>, base:&GameObject, game_state:&mut GameState){
    let landed_data_node = base.get("landed_data");
    if landed_data_node.is_some(){
        let landed_data = landed_data_node.unwrap().as_object().unwrap();
        let dread_node = landed_data.get("dread");
        if dread_node.is_some(){
            *dread = dread_node.unwrap().as_string().parse::<f32>().unwrap();
        }
        let strength_node = landed_data.get("strength");
        if strength_node.is_some(){
            *strength = strength_node.unwrap().as_string().parse::<f32>().unwrap();
        }
        let titles_node = landed_data.get("domain");
        if titles_node.is_some(){
            for t in titles_node.unwrap().as_object().unwrap().get_array_iter(){
                titles.push(game_state.get_title(&t.as_id()));
            }
        }
        let vassals_node = landed_data.get("vassal_contracts");
        if vassals_node.is_some(){
            for v in vassals_node.unwrap().as_object().unwrap().get_array_iter(){
                vassals.push(game_state.get_vassal(&v.as_id()));
            }
        }
    }
}

/// Gets the dynasty of the character
fn get_dynasty(base:&GameObject, game_state:&mut GameState) -> Option<Shared<Dynasty>>{
    let dynasty_id = base.get("dynasty_house");
    if dynasty_id.is_some(){
        let d = game_state.get_dynasty(&dynasty_id.unwrap().as_id());
        d.get_internal_mut().register_member();
        return Some(d);
    }
    None
}

impl Character {
    /// Gets the faith of the character
    pub fn get_faith(&self) -> Option<Shared<Faith>> {
        if self.faith.is_some(){
            return Some(self.faith.as_ref().unwrap().clone());
        }
        None
    }

    /// Gets the culture of the character
    pub fn get_culture(&self) -> Option<Shared<Culture>> {
        if self.culture.is_some(){
            return Some(self.culture.as_ref().unwrap().clone());
        }
        None
    }

    /// Adds a character as a parent of this character
    pub fn register_parent(&mut self, parent:Shared<Character>){
        self.parents.push(parent);
    }

    /// Gets the death date string of the character
    pub fn get_death_date(&self) -> Option<GameString> {
        self.date.clone()
    }

    /// Gets the iterator of the children of the character
    pub fn get_children_iter(&self) ->  Iter<Shared<Character>>{
        self.children.iter()
    }
}

impl DummyInit for Character{
    fn dummy(id:GameId) -> Self {
        Character{
            name: None,
            nick: None,
            birth: None,
            dead: false,
            date: None,
            reason: None,
            faith: None,
            culture: None,
            house: None,
            skills: Vec::new(),
            traits: Vec::new(),
            recessive: Vec::new(),
            spouses: Vec::new(),
            former: Vec::new(),
            children: Vec::new(),
            parents: Vec::new(),
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
            female: false,
            id: id,
            depth: 0,
            localized:false,
            name_localized:false
        }
    }

    fn init(&mut self, base:&GameObject, game_state:&mut GameState) {
        let female = base.get("female");
        if female.is_some(){
            self.female = female.unwrap().as_string().as_str() == "yes";
        }
        get_dead(&mut self.dead, &mut self.reason, &mut self.date, &mut self.titles, &base, game_state);
        //find skills
        get_skills(&mut self.skills, &base);
        //find recessive traits
        get_recessive(&mut self.recessive, &base);
        get_family(self.id, &mut self.spouses, &mut self.former, &mut self.children, &base, game_state);
        //find dna
        let dna = match base.get("dna"){
            Some(d) => Some(d.as_string()),
            None => None
        };
        //find traits
        get_traits(&mut self.traits, &base, game_state);
        //find alive data
        if !self.dead {
            parse_alive_data(&base, &mut self.piety, &mut self.prestige, &mut self.gold, &mut self.kills, &mut self.languages, &mut self.traits, &mut self.memories, game_state);
        }
        //find landed data
        get_landed_data(&mut self.dread, &mut self.strength, &mut self.titles, &mut self.vassals, &base, game_state);
        //find house
        let house = get_dynasty(&base, game_state);
        self.name = Some(base.get("first_name").unwrap().as_string());
        self.nick = base.get("nickname").map(|v| v.as_string());
        self.birth = Some(base.get("birth").unwrap().as_string());
        self.house = house.clone();
        self.faith = get_faith(&base, game_state);
        self.culture = get_culture(&base, game_state);
        self.dna = dna;
    }
}

impl GameObjectDerived for Character {
    fn get_id(&self) -> GameId {
        self.id
    }

    fn get_name(&self) -> GameString {
        if self.name.is_none(){
            return GameString::wrap("Unknown".to_string())
        }
        self.name.as_ref().unwrap().clone()
    }
}

impl Serialize for Character {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Character", 28)?;
        state.serialize_field("name", &self.get_name())?;
        state.serialize_field("nick", &self.nick)?;
        state.serialize_field("birth", &self.birth)?;
        state.serialize_field("dead", &self.dead)?;
        state.serialize_field("date", &self.date)?;
        state.serialize_field("reason", &self.reason)?;
        let mut faith = self.faith.clone();
        let mut culture = self.culture.clone();
        if self.house.is_some(){
            let h = self.house.as_ref().unwrap();
            let rd = DerivedRef::<Dynasty>::from_derived(h.clone());
            state.serialize_field("house", &rd)?;
            if faith.is_none(){
                let o = h.get_internal();
                faith = o.get_faith();
            }
            if culture.is_none(){
                let o = h.get_internal();
                culture = o.get_culture();
            }
        }
        if faith.is_some(){
            let rf = DerivedRef::<Faith>::from_derived(faith.as_ref().unwrap().clone());
            state.serialize_field("faith", &rf)?;
        }
        if culture.is_some(){
            let rc = DerivedRef::<Culture>::from_derived(culture.as_ref().unwrap().clone());
            state.serialize_field("culture", &rc)?;
        }
        state.serialize_field("skills", &self.skills)?;
        state.serialize_field("traits", &self.traits)?;
        state.serialize_field("recessive", &self.recessive)?;
        //serialize spouses as DerivedRef
        let spouses = serialize_array::<Character>(&self.spouses);
        state.serialize_field("spouses", &spouses)?;
        //serialize former as DerivedRef
        let former = serialize_array::<Character>(&self.former);
        state.serialize_field("former", &former)?;
        //serialize children as DerivedRef
        let children = serialize_array::<Character>(&self.children);
        state.serialize_field("children", &children)?;
        //serialize parents as DerivedRef
        let parents = serialize_array::<Character>(&self.parents);
        state.serialize_field("parents", &parents)?;
        state.serialize_field("dna", &self.dna)?;
        //serialize memories as DerivedRef
        state.serialize_field("memories", &self.memories)?;
        //serialize titles as DerivedRef
        let titles = serialize_array::<Title>(&self.titles);
        state.serialize_field("titles", &titles)?;
        state.serialize_field("gold", &self.gold)?;
        state.serialize_field("piety", &self.piety)?;
        state.serialize_field("prestige", &self.prestige)?;
        state.serialize_field("dread", &self.dread)?;
        state.serialize_field("strength", &self.strength)?;
        //serialize kills as DerivedRef
        let kills = serialize_array::<Character>(&self.kills);
        state.serialize_field("kills", &kills)?;
        state.serialize_field("languages", &self.languages)?;
        //vassals is already serialized as DerivedRef
        state.serialize_field("vassals", &self.vassals)?;
        state.serialize_field("id", &self.id)?;
        state.end()
    }
}

impl Renderable for Character {
    fn get_context(&self) -> minijinja::Value {
        return context!{character=>self};
    }

    fn get_template() -> &'static str {
        "charTemplate.html"
    }

    fn get_subdir() -> &'static str {
        "characters"
    }

    fn render_all(&self, renderer: &mut Renderer, game_map:Option<&GameMap>, grapher: Option<&Grapher>){
        if !renderer.render(self){
            return;
        }
        if self.faith.is_some(){
            self.faith.as_ref().unwrap().get_internal().render_all(renderer, game_map, grapher);
        }
        if self.culture.is_some(){
            self.culture.as_ref().unwrap().get_internal().render_all(renderer, game_map, grapher);
        }
        if self.house.is_some(){
            self.house.as_ref().unwrap().get_internal().render_all(renderer, game_map, grapher);
        }
        for s in self.spouses.iter(){
            s.get_internal().render_all(renderer, game_map, grapher);
        }
        for s in self.former.iter(){
            s.get_internal().render_all(renderer, game_map, grapher);
        }
        for s in self.children.iter(){
            s.get_internal().render_all(renderer, game_map, grapher);
        }
        for s in self.parents.iter(){
            s.get_internal().render_all(renderer, game_map, grapher);
        }
        for s in self.kills.iter(){
            s.get_internal().render_all(renderer, game_map, grapher);
        }
        for s in self.vassals.iter(){
            s.get_internal().get_ref().get_internal().render_all(renderer, game_map, grapher);
        }
        for s in self.titles.iter(){
            s.get_internal().render_all(renderer, game_map, grapher);
        }
        for m in self.memories.iter() {
            m.get_internal().render_participants(renderer, game_map, grapher);
        }
    }
}

impl Cullable for Character {
    fn set_depth(&mut self, depth:usize, localization:&Localizer) {
        if depth <= self.depth && depth != 0{
            return;
        }
        if !self.name_localized {
            if self.name.is_none() {
                self.name = Some(GameString::wrap("Unknown".to_string()));
            }
            else{
                self.name = Some(localization.localize(self.name.as_ref().unwrap().as_str()));
            }
            self.name_localized = true;
        }
        if depth == 0 {
            return;
        }
        if !self.localized {
            if self.nick.is_some(){
                self.nick = Some(localization.localize(self.nick.as_ref().unwrap().as_str()));
            }
            if self.reason.is_some(){
                self.reason = Some(localization.localize(self.reason.as_ref().unwrap().as_str()));
            }
            for t in self.traits.iter_mut(){
                *t = localization.localize(t.as_str());
            }
            /* 
            for t in self.recessive.iter_mut(){
                *t = localization.localize(t.as_str());
            }
            */
            for t in self.languages.iter_mut(){
                *t = localization.localize(t.as_str());
            }
            self.localized = true;
        }
        //cullable set
        self.depth = depth;
        for s in self.spouses.iter(){
            let o = s.try_get_internal_mut();
            if o.is_ok(){
                o.unwrap().set_depth(depth - 1, localization);
            }
        }
        for s in self.former.iter(){
            let o = s.try_get_internal_mut();
            if o.is_ok(){
                o.unwrap().set_depth(depth - 1, localization);
            }
        }
        for s in self.children.iter(){
            let o = s.try_get_internal_mut();
            if o.is_ok(){
                o.unwrap().set_depth(depth - 1, localization);
            }
        }
        for s in self.parents.iter(){
            let o = s.try_get_internal_mut();
            if o.is_ok(){
                o.unwrap().set_depth(depth - 1, localization);
            }
        }
        for s in self.kills.iter(){
            let o = s.try_get_internal_mut();
            if o.is_ok(){
                o.unwrap().set_depth(depth - 1, localization);
            }
        }
        for s in self.vassals.iter(){
            let o = s.try_get_internal_mut();
            if o.is_ok(){
                o.unwrap().get_ref().get_internal_mut().set_depth(depth - 1, localization);
            }
        }
        if self.culture.is_some(){
            let o = self.culture.as_ref().unwrap().try_get_internal_mut();
            if o.is_ok(){
                o.unwrap().set_depth(depth - 1, localization);
            }
        }
        if self.faith.is_some(){
            let o = self.faith.as_ref().unwrap().try_get_internal_mut();
            if o.is_ok(){
                o.unwrap().set_depth(depth - 1, localization);
            }
        }
        for s in self.titles.iter(){
            let o = s.try_get_internal_mut();
            if o.is_ok(){
                o.unwrap().set_depth(depth - 1, localization);
            }
        }
        for s in self.memories.iter(){
            s.get_internal_mut().set_depth(depth - 1, localization);
        }
        if self.house.is_some(){
            let o = self.house.as_ref().unwrap().try_get_internal_mut();
            if o.is_ok(){
                o.unwrap().set_depth(depth - 1, localization);
            }
        }
    }

    fn get_depth(&self) -> usize {
        self.depth
    }
}
