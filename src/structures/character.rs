use jomini::common::Date;
use serde::Serialize;

use super::{
    super::{
        display::{Renderable, TreeNode},
        game_data::{Localizable, LocalizationError, Localize},
        jinja_env::C_TEMPLATE_NAME,
        parser::{
            GameObjectMap, GameObjectMapping, GameState, GameString, ParsingError, SaveFileValue,
        },
        types::{OneOrMany, Wrapper, WrapperMut},
    },
    Artifact, Culture, DummyInit, Dynasty, Faith, GameId, GameObjectDerived, GameObjectDerivedType,
    Memory, Shared, Title,
};

/// An enum that holds either a character or a reference to a character.
/// Effectively either a vassal([Character]) or a vassal([DerivedRef]) contract.
/// This is done so that we can hold a reference to a vassal contract, and also manually added characters from vassals registering themselves via [Character::add_vassal].
#[derive(Serialize, Debug)]
#[serde(untagged)]
enum Vassal {
    Character(Shared<Character>),
    Reference(Shared<Option<Shared<Character>>>),
}

// MAYBE enum for dead and alive character?

/// Represents a character in the game.
/// Implements [GameObjectDerived], [Renderable] and [Cullable].
#[derive(Serialize, Debug)]
pub struct Character {
    id: GameId,
    name: Option<GameString>,
    nick: Option<GameString>,
    birth: Option<Date>,
    dead: bool,
    date: Option<Date>,
    reason: Option<GameString>,
    #[serde(default = "Character::get_faith")]
    faith: Option<Shared<Faith>>,
    #[serde(default = "Character::get_culture")]
    culture: Option<Shared<Culture>>,
    house: Option<Shared<Dynasty>>,
    skills: Vec<i8>,
    traits: Vec<GameString>,
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
    vassals: Vec<Vassal>,
    liege: Option<Shared<Character>>,
    female: bool,
    artifacts: Vec<Shared<Artifact>>,
}

// So both faith and culture can be stored for a character in the latest leader of their house.
// The problem with reading that now is that while Houses are already likely loaded,
// the characters that the houses hold reference to are likely still dummy, so we can't read the faith and culture from the house leader.
// So we will be returning None for now in case either is missing, but later during serialization read the one from house.

/// Processes a currency node of the character
fn process_currency(currency_node: Option<&SaveFileValue>) -> Result<f32, ParsingError> {
    if let Some(o) = currency_node {
        if let Some(currency) = o.as_object()?.as_map()?.get("accumulated") {
            return Ok(currency.as_real()? as f32);
        } else {
            return Ok(0.0);
        }
    } else {
        return Ok(0.0);
    }
}

impl Character {
    /// Gets whether the character is female
    pub fn get_female(&self) -> bool {
        self.female
    }

    /// Gets the faith of the character
    pub fn get_faith(&self) -> Option<Shared<Faith>> {
        if let Some(faith) = &self.faith {
            return Some(faith.clone());
        } else {
            if let Some(house) = &self.house {
                let o = house.get_internal();
                let leader = o.get_leader();
                let leader = leader.get_internal();
                if leader.get_id() != self.id {
                    return leader.get_faith();
                }
            }
        }
        None
    }

    /// Gets the culture of the character
    pub fn get_culture(&self) -> Option<Shared<Culture>> {
        if let Some(culture) = &self.culture {
            return Some(culture.clone());
        } else {
            if let Some(house) = &self.house {
                let o = house.get_internal();
                let leader = o.get_leader();
                let leader = leader.get_internal();
                if leader.get_id() != self.id {
                    return leader.get_culture();
                }
            }
        }
        None
    }

    /// Adds a character as a parent of this character
    pub fn register_parent(&mut self, parent: Shared<Character>) {
        self.parents.push(parent);
    }

    /// Gets the death date string of the character
    pub fn get_death_date(&self) -> Option<Date> {
        self.date.clone()
    }

    /// Adds a character as a vassal of this character
    pub fn add_vassal(&mut self, vassal: Shared<Character>) {
        self.vassals.push(Vassal::Character(vassal));
    }

    /* TODO implement this.
    The problem is that we cannot do this in the init, because the vassals aren't loaded, and within cull we don't have a shared reference

    pub fn set_liege(&mut self, liege:Shared<Character>){
        self.liege = Some(DerivedRef::from_derived(liege));
    }
    */

    /// Gets all of the held de jure barony keys of the character and their vassals
    pub fn get_barony_keys(&self, de_jure: bool) -> Vec<GameString> {
        let mut provinces = Vec::new();
        for title in self.titles.iter() {
            let title = title.get_internal();
            let key = title.get_key().unwrap();
            if key.starts_with("e_") || key.starts_with("k_") {
                //for kingdoms and empires we don't want to add the de jure baronies
                continue;
            } else {
                if de_jure {
                    provinces.append(&mut title.get_de_jure_barony_keys());
                } else {
                    provinces.append(&mut title.get_barony_keys());
                }
            }
        }
        for vassal in self.vassals.iter() {
            match vassal {
                Vassal::Character(c) => {
                    provinces.append(&mut c.get_internal().get_barony_keys(de_jure))
                }
                Vassal::Reference(c) => provinces.append(
                    &mut c
                        .get_internal()
                        .as_ref()
                        .unwrap()
                        .get_internal()
                        .get_barony_keys(de_jure),
                ),
            }
        }
        return provinces;
    }

    /// Gets the descendants of the character
    pub fn get_descendants(&self) -> Vec<Shared<Character>> {
        let mut res = Vec::new();
        let mut stack: Vec<Shared<Character>> = Vec::new();
        for child in self.children.iter() {
            stack.push(child.clone());
            res.push(child.clone());
        }
        while !stack.is_empty() {
            let c = stack.pop().unwrap();
            let c = c.get_internal();
            for child in c.children.iter() {
                stack.push(child.clone());
                res.push(child.clone());
            }
        }
        return res;
    }

    /// Gets the dynasty of the character
    pub fn get_dynasty(&self) -> Option<Shared<Dynasty>> {
        if let Some(house) = &self.house {
            return Some(house.clone());
        } else {
            return None;
        }
    }
}

impl DummyInit for Character {
    fn dummy(id: GameId) -> Self {
        Character {
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
            liege: None,
            female: false,
            id: id,
            artifacts: Vec::new(),
        }
    }

    fn init(
        &mut self,
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<(), ParsingError> {
        if let Some(female) = base.get("female") {
            self.female = female.as_boolean()?;
        }
        if let Some(dead_data) = base.get("dead_data") {
            self.dead = true;
            let o = dead_data.as_object()?.as_map()?;
            if let Some(reason_node) = o.get("reason") {
                self.reason = Some(reason_node.as_string()?);
            }
            if let Some(domain_node) = o.get("domain") {
                for t in domain_node.as_object()?.as_array()? {
                    self.titles.push(game_state.get_title(&t.as_id()?));
                }
            }
            self.date = Some(o.get_date("date")?);
            if let Some(liege_node) = o.get("liege") {
                let id = liege_node.as_id()?;
                if id != self.id {
                    let liege_chr = game_state.get_character(&id).clone();
                    if let Ok(mut o) = liege_chr.try_get_internal_mut() {
                        o.add_vassal(game_state.get_character(&self.id));
                    }
                    self.liege = Some(liege_chr.clone());
                }
            }
        }
        //find skills
        for s in base.get_object("skill")?.as_array()?.into_iter() {
            self.skills.push(s.as_integer()? as i8);
        }
        if let Some(family_data) = base.get("family_data") {
            let f = family_data.as_object()?;
            if !f.is_empty() {
                let f = f.as_map()?;
                if let Some(former_spouses_node) = f.get("former_spouses") {
                    for s in former_spouses_node.as_object()?.as_array()? {
                        self.former
                            .push(game_state.get_character(&s.as_id()?).clone());
                    }
                }
                if let Some(spouse_node) = f.get("spouse") {
                    if let SaveFileValue::Object(o) = spouse_node {
                        for s in o.as_array()? {
                            let c = game_state.get_character(&s.as_id()?).clone();
                            let contains = self
                                .former
                                .iter()
                                .any(|x| x.get_internal().get_id() == c.get_internal().get_id());
                            if !contains {
                                self.spouses.push(c);
                            }
                        }
                    } else {
                        let c = game_state.get_character(&spouse_node.as_id()?).clone();
                        let contains = self
                            .former
                            .iter()
                            .any(|x| x.get_internal().get_id() == c.get_internal().get_id());
                        if !contains {
                            self.spouses.push(c);
                        }
                    }
                }
                if let Some(primary_spouse_node) = f.get("primary_spouse") {
                    let c = game_state
                        .get_character(&primary_spouse_node.as_id()?)
                        .clone();
                    let mut contains = self
                        .spouses
                        .iter()
                        .any(|x| x.get_internal().get_id() == c.get_internal().get_id());
                    contains = contains
                        || self
                            .spouses
                            .iter()
                            .any(|x| x.get_internal().get_id() == c.get_internal().get_id());
                    if !contains {
                        self.spouses.push(c);
                    }
                }
                if let Some(children_node) = f.get("child") {
                    let parent = game_state.get_character(&self.id);
                    for s in children_node.as_object()?.as_array()? {
                        let c = game_state.get_character(&s.as_id()?).clone();
                        c.get_internal_mut().register_parent(parent.clone());
                        self.children.push(c);
                    }
                }
            }
        }
        //find dna
        if let Some(dna) = base.get("dna") {
            self.dna = Some(dna.as_string()?);
        }
        //find traits
        if let Some(traits_node) = base.get("traits") {
            for t in traits_node.as_object()?.as_array()? {
                let index = t.as_integer()? as u16;
                self.traits.push(game_state.get_trait(index));
            }
        }
        //find alive data
        if !self.dead {
            if let Some(alive_data) = base.get("alive_data") {
                let alive_data = alive_data.as_object()?.as_map()?;
                self.piety = process_currency(alive_data.get("piety"))?;
                self.prestige = process_currency(alive_data.get("prestige"))?;
                if let Some(kills_node) = alive_data.get("kills") {
                    for k in kills_node.as_object()?.as_array()? {
                        self.kills
                            .push(game_state.get_character(&k.as_id()?).clone());
                    }
                }
                if let Some(gold_node) = alive_data.get("gold") {
                    self.gold = gold_node.as_real()? as f32;
                }
                for l in alive_data.get_object("languages")?.as_array()? {
                    self.languages.push(l.as_string()?);
                }
                if let Some(perk_node) = alive_data.get("perks") {
                    for p in perk_node.as_object()?.as_array()? {
                        self.traits.push(p.as_string()?);
                    }
                }
                if let Some(memory_node) = alive_data.get("memories") {
                    for m in memory_node.as_object()?.as_array()? {
                        self.memories
                            .push(game_state.get_memory(&m.as_id()?).clone());
                    }
                }
                if let Some(inventory_node) = alive_data.get("inventory") {
                    if let Some(artifacts_node) =
                        inventory_node.as_object()?.as_map()?.get("artifacts")
                    {
                        for a in artifacts_node.as_object()?.as_array()? {
                            self.artifacts
                                .push(game_state.get_artifact(&a.as_id()?).clone());
                        }
                    }
                }
            }
        }
        //find landed data
        if let Some(landed_data_node) = base.get("landed_data") {
            let landed_data = landed_data_node.as_object()?.as_map()?;
            if let Some(dread_node) = landed_data.get("dread") {
                self.dread = dread_node.as_real()? as f32;
            }
            if let Some(strength_node) = landed_data.get("strength") {
                self.strength = strength_node.as_real()? as f32;
            }
            if let Some(titles_node) = landed_data.get("domain") {
                for t in titles_node.as_object()?.as_array()? {
                    self.titles.push(game_state.get_title(&t.as_id()?));
                }
            }
            if let Some(vassals_node) = landed_data.get("vassal_contracts") {
                for v in vassals_node.as_object()?.as_array()? {
                    self.vassals
                        .push(Vassal::Reference(game_state.get_vassal(&v.as_id()?)));
                }
            }
        }
        //find house
        self.name = Some(base.get_string("first_name")?);
        if let Some(text) = base.get("nickname_text").or(base.get("nickname")) {
            self.nick = Some(text.as_string()?);
        }
        self.birth = Some(base.get_date("birth")?);
        if let Some(dynasty_id) = base.get("dynasty_house") {
            let d = game_state.get_dynasty(&dynasty_id.as_id()?);
            d.get_internal_mut()
                .register_member(game_state.get_character(&self.id));
            self.house = Some(d);
        }
        if let Some(faith_node) = base.get("faith") {
            self.faith = Some(game_state.get_faith(&faith_node.as_id()?))
        }
        if let Some(culture_node) = base.get("culture") {
            self.culture = Some(game_state.get_culture(&culture_node.as_id()?).clone());
        }
        Ok(())
    }
}

impl GameObjectDerived for Character {
    fn get_id(&self) -> GameId {
        self.id
    }

    fn get_name(&self) -> GameString {
        if let Some(name) = &self.name {
            return name.clone();
        } else {
            return GameString::from("Unknown".to_string());
        }
    }

    fn get_references<E: From<GameObjectDerivedType>, C: Extend<E>>(&self, collection: &mut C) {
        if let Some(faith) = &self.faith {
            collection.extend([E::from(faith.clone().into())]);
        }
        if let Some(culture) = &self.culture {
            collection.extend([E::from(culture.clone().into())]);
        }
        if let Some(house) = &self.house {
            collection.extend([E::from(house.clone().into())]);
        }
        if let Some(liege) = &self.liege {
            collection.extend([E::from(liege.clone().into())]);
        }
        for s in self.spouses.iter() {
            collection.extend([E::from(s.clone().into())]);
        }
        for s in self.former.iter() {
            collection.extend([E::from(s.clone().into())]);
        }
        for s in self.children.iter() {
            collection.extend([E::from(s.clone().into())]);
        }
        for s in self.parents.iter() {
            collection.extend([E::from(s.clone().into())]);
        }
        for s in self.kills.iter() {
            collection.extend([E::from(s.clone().into())]);
        }
        for s in self.vassals.iter() {
            match s {
                Vassal::Character(c) => collection.extend([E::from(c.clone().into())]),
                Vassal::Reference(c) => {
                    collection.extend([E::from(c.get_internal().as_ref().unwrap().clone().into())])
                }
            }
        }
        for s in self.titles.iter() {
            collection.extend([E::from(s.clone().into())]);
        }
        for m in self.memories.iter() {
            collection.extend([E::from(m.clone().into())]);
        }
        for a in self.artifacts.iter() {
            collection.extend([E::from(a.clone().into())]);
        }
    }
}

impl TreeNode for Character {
    fn get_children(&self) -> Option<OneOrMany<Character>> {
        if self.children.is_empty() {
            return None;
        }
        Some(OneOrMany::Many(&self.children))
    }

    fn get_parent(&self) -> Option<OneOrMany<Character>> {
        if self.parents.is_empty() {
            return None;
        }
        Some(OneOrMany::Many(&self.parents))
    }

    fn get_class(&self) -> Option<GameString> {
        if let Some(house) = &self.house {
            return Some(house.get_internal().get_name().clone());
        } else {
            None
        }
    }
}

impl Renderable for Character {
    fn get_template() -> &'static str {
        C_TEMPLATE_NAME
    }

    fn get_subdir() -> &'static str {
        "characters"
    }
}

impl Localizable for Character {
    fn localize<L: Localize<GameString>>(
        &mut self,
        localization: &mut L,
    ) -> Result<(), LocalizationError> {
        if self.name.is_none() {
            return Ok(());
        } else {
            self.name = Some(localization.localize(self.name.as_ref().unwrap())?);
        }
        if let Some(nick) = &self.nick {
            self.nick = Some(localization.localize(nick)?);
        }
        if let Some(reason) = &self.reason {
            self.reason = Some(localization.localize_query(reason, |stack| {
                if stack.len() == 2 {
                    if stack[0].0 == "GetTrait" {
                        return localization
                            .localize("trait_".to_string() + &stack[0].1[0])
                            .ok();
                    } else if stack[1].0 == "GetHerHis" {
                        if self.female {
                            return Some("her".into());
                        } else {
                            return Some("his".into());
                        }
                    } else if stack[1].0 == "GetHerselfHimself" {
                        if self.female {
                            return Some("herself".into());
                        } else {
                            return Some("himself".into());
                        }
                    }
                }
                None
            })?);
        }
        for t in self.traits.iter_mut() {
            let key = if t.starts_with("child_of_concubine") {
                "trait_child_of_concubine".to_string()
            } else if t.ends_with("hajjaj") {
                if self.female {
                    "trait_hajjah".to_string()
                } else {
                    "trait_hajji".to_string()
                }
            } else if t.as_ref() == "lifestyle_traveler" {
                // TODO this should reflect the traveler level? i think
                "trait_traveler_1".to_string()
            } else if t.starts_with("viking") {
                // TODO viking should be displayed if the culture has longships (trait_viking_has_longships)
                "trait_viking_fallback".to_string()
            } else if t.starts_with("shieldmaiden") {
                if self.female {
                    "trait_shieldmaiden_female".to_string()
                } else {
                    "trait_shieldmaiden_male".to_string()
                }
            } else {
                "trait_".to_string() + t
            };
            *t = localization.localize(key)?;
        }
        for t in self.languages.iter_mut() {
            *t = localization.localize(t.to_string() + "_name")?;
        }
        Ok(())
    }
}
