use std::collections::HashSet;

use jomini::common::Date;
use serde::Serialize;

use super::{
    super::{
        display::{ProceduralPath, Renderable, TreeNode},
        game_data::{GameData, Localizable, LocalizationError, Localize},
        jinja_env::C_TEMPLATE_NAME,
        parser::{GameObjectMap, GameObjectMapping, GameState, ParsingError, SaveFileValue},
        types::{GameString, Shared, Wrapper, WrapperMut},
    },
    Artifact, Culture, EntityRef, Faith, FromGameObject, GameObjectDerived, GameObjectEntity,
    GameRef, House, Memory, Title,
};

/// An enum that holds either a character or a reference to a character.
/// Effectively either a vassal([Character]) or a vassal contract.
/// This is done so that we can hold a reference to a vassal contract, and also manually added characters from vassals registering themselves via [Character::add_vassal].
#[derive(Serialize, Clone)]
#[serde(untagged)]
enum Vassal {
    Character(Shared<GameObjectEntity<Character>>),
    Reference(Shared<Option<Shared<GameObjectEntity<Character>>>>),
}

// MAYBE enum for dead and alive character?

/// Represents a character in the game.
#[derive(Serialize, Clone)]
pub struct Character {
    name: GameString,
    nick: Option<GameString>,
    birth: Date,
    dead: bool,
    date: Option<Date>,
    reason: Option<GameString>,
    faith: Option<GameRef<Faith>>,
    culture: Option<GameRef<Culture>>,
    house: Option<GameRef<House>>,
    skills: Vec<i8>,
    traits: Vec<GameString>,
    spouses: HashSet<GameRef<Character>>,
    former: Vec<GameRef<Character>>,
    children: Vec<GameRef<Character>>,
    parents: Vec<GameRef<Character>>,
    dna: Option<GameString>,
    memories: Vec<GameRef<Memory>>,
    titles: Vec<GameRef<Title>>,
    gold: f32,
    piety: f32,
    prestige: f32,
    dread: f32,
    strength: f32,
    kills: Vec<GameRef<Character>>,
    languages: Vec<GameString>,
    vassals: Vec<Vassal>,
    liege: Option<GameRef<Character>>,
    female: bool,
    artifacts: Vec<GameRef<Artifact>>,
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

    pub fn get_faith(&self) -> Option<GameRef<Faith>> {
        self.faith.clone()
    }

    pub fn get_culture(&self) -> Option<GameRef<Culture>> {
        self.culture.clone()
    }

    /// Adds a character as a parent of this character
    pub fn register_parent(&mut self, parent: GameRef<Character>) {
        self.parents.push(parent);
    }

    /// Gets the death date string of the character
    pub fn get_death_date(&self) -> Option<Date> {
        self.date.clone()
    }

    /// Adds a character as a vassal of this character
    pub fn add_vassal(&mut self, vassal: GameRef<Character>) {
        self.vassals.push(Vassal::Character(vassal));
    }

    pub fn set_liege(&mut self, liege: GameRef<Character>) {
        self.liege = Some(liege);
    }

    /// Gets all of the held de jure barony keys of the character and their vassals
    pub fn get_barony_keys(&self, de_jure: bool) -> Vec<GameString> {
        let mut provinces = Vec::new();
        for title in self.titles.iter() {
            if let Some(title) = title.get_internal().inner() {
                let key = title.get_key();
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
        }
        for vassal in self.vassals.iter() {
            match vassal {
                Vassal::Character(c) => {
                    if let Some(c) = c.get_internal().inner() {
                        provinces.append(&mut c.get_barony_keys(de_jure));
                    }
                }
                Vassal::Reference(c) => {
                    if let Some(c) = c.get_internal().as_ref() {
                        if let Some(c) = c.get_internal().inner() {
                            provinces.append(&mut c.get_barony_keys(de_jure))
                        }
                    }
                }
            }
        }
        return provinces;
    }

    /// Gets the descendants of the character
    pub fn get_descendants(&self) -> Vec<GameRef<Character>> {
        let mut res = Vec::new();
        let mut stack: Vec<GameRef<Character>> = Vec::new();
        for child in self.children.iter() {
            stack.push(child.clone());
            res.push(child.clone());
        }
        while let Some(c) = stack.pop() {
            if let Some(c) = c.get_internal().inner() {
                for child in c.children.iter() {
                    stack.push(child.clone());
                    res.push(child.clone());
                }
            }
        }
        return res;
    }

    /// Gets the dynasty of the character
    pub fn get_house(&self) -> Option<GameRef<House>> {
        if let Some(house) = &self.house {
            return Some(house.clone());
        } else {
            return None;
        }
    }
}

impl FromGameObject for Character {
    fn from_game_object(
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError> {
        let mut val = Self {
            // a few simple keys
            female: base
                .get("female")
                .map_or(Ok(false), |val| val.as_boolean())?,
            name: base.get_string("first_name")?,
            birth: base.get_date("birth")?,
            // non mandatory, yet simple keys
            nick: base
                .get("nickname_text")
                .or(base.get("nickname"))
                .map(|x| x.as_string())
                .transpose()?,
            faith: base
                .get("faith")
                .map(|x| x.as_id().and_then(|id| Ok(game_state.get_faith(&id))))
                .transpose()?,
            culture: base
                .get("culture")
                .map(|x| x.as_id().and_then(|id| Ok(game_state.get_culture(&id))))
                .transpose()?,
            dna: base.get("dna").map(|x| x.as_string()).transpose()?,
            traits: Vec::new(),
            skills: Vec::new(),
            // keys grouped together in sections, we resolve these later
            dead: false,
            date: None,
            reason: None,
            house: None,
            spouses: HashSet::new(),
            former: Vec::new(),
            children: Vec::new(),
            parents: Vec::new(),
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
            artifacts: Vec::new(),
        };
        for s in base.get_object("skill")?.as_array()?.into_iter() {
            val.skills.push(s.as_integer()? as i8);
        }
        if let Some(traits_node) = base.get("traits") {
            for t in traits_node.as_object()?.as_array()? {
                let integer = t.as_integer()?;
                if integer >= 0 {
                    // WTF? a negative index? ok schizo save file go back to bed
                    val.traits.push(game_state.get_trait(integer as u16));
                }
            }
        }
        if let Some(dynasty_id) = base.get("dynasty_house") {
            val.house = Some(game_state.get_house(&dynasty_id.as_id()?));
        }
        if let Some(landed_data_node) = base.get("landed_data") {
            let landed_data = landed_data_node.as_object()?.as_map()?;
            if let Some(dread_node) = landed_data.get("dread") {
                val.dread = dread_node.as_real()? as f32;
            }
            if let Some(strength_node) = landed_data.get("strength") {
                val.strength = strength_node.as_real()? as f32;
            }
            if let Some(titles_node) = landed_data.get("domain") {
                for t in titles_node.as_object()?.as_array()? {
                    val.titles.push(game_state.get_title(&t.as_id()?));
                }
            }
            if let Some(vassals_node) = landed_data.get("vassal_contracts") {
                for v in vassals_node.as_object()?.as_array()? {
                    val.vassals
                        .push(Vassal::Reference(game_state.get_vassal(&v.as_id()?)));
                }
            }
        }
        if let Some(dead_data) = base.get("dead_data") {
            val.dead = true;
            let o = dead_data.as_object()?.as_map()?;
            if let Some(reason_node) = o.get("reason") {
                val.reason = Some(reason_node.as_string()?);
            }
            if let Some(domain_node) = o.get("domain") {
                for t in domain_node.as_object()?.as_array()? {
                    val.titles.push(game_state.get_title(&t.as_id()?));
                }
            }
            val.date = Some(o.get_date("date")?);
            if let Some(liege_node) = o.get("liege") {
                val.liege = Some(game_state.get_character(&liege_node.as_id()?));
            }
            if let Some(memory_node) = o.get("memories") {
                for m in memory_node.as_object()?.as_array()? {
                    val.memories
                        .push(game_state.get_memory(&m.as_id()?).clone());
                }
            }
        } else if let Some(alive_data) = base.get("alive_data") {
            val.dead = false;
            let alive_data = alive_data.as_object()?.as_map()?;
            val.piety = process_currency(alive_data.get("piety"))?;
            val.prestige = process_currency(alive_data.get("prestige"))?;
            if let Some(kills_node) = alive_data.get("kills") {
                for k in kills_node.as_object()?.as_array()? {
                    val.kills
                        .push(game_state.get_character(&k.as_id()?).clone());
                }
            }
            if let Some(gold_node) = alive_data.get("gold") {
                val.gold = gold_node.as_real()? as f32;
            }
            for l in alive_data.get_object("languages")?.as_array()? {
                val.languages.push(l.as_string()?);
            }
            if let Some(perk_node) = alive_data.get("perks") {
                for p in perk_node.as_object()?.as_array()? {
                    val.traits.push(p.as_string()?);
                }
            }
            if let Some(memory_node) = alive_data.get("memories") {
                for m in memory_node.as_object()?.as_array()? {
                    val.memories
                        .push(game_state.get_memory(&m.as_id()?).clone());
                }
            }
            if let Some(inventory_node) = alive_data.get("inventory") {
                if let Some(artifacts_node) = inventory_node.as_object()?.as_map()?.get("artifacts")
                {
                    for a in artifacts_node.as_object()?.as_array()? {
                        val.artifacts
                            .push(game_state.get_artifact(&a.as_id()?).clone());
                    }
                }
            }
        }
        if let Some(family_data) = base.get("family_data") {
            let f = family_data.as_object()?;
            if !f.is_empty() {
                let f = f.as_map()?;
                if let Some(former_spouses_node) = f.get("former_spouses") {
                    for s in former_spouses_node.as_object()?.as_array()? {
                        val.former
                            .push(game_state.get_character(&s.as_id()?).clone());
                    }
                }
                if let Some(spouse_node) = f.get("spouse") {
                    if let SaveFileValue::Object(o) = spouse_node {
                        for s in o.as_array()? {
                            let c = game_state.get_character(&s.as_id()?).clone();
                            val.spouses.insert(c);
                        }
                    } else {
                        let c = game_state.get_character(&spouse_node.as_id()?).clone();
                        val.spouses.insert(c);
                    }
                }
                if let Some(primary_spouse_node) = f.get("primary_spouse") {
                    let c = game_state
                        .get_character(&primary_spouse_node.as_id()?)
                        .clone();
                    val.spouses.insert(c);
                }
                if let Some(children_node) = f.get("child") {
                    for s in children_node.as_object()?.as_array()? {
                        val.children
                            .push(game_state.get_character(&s.as_id()?).clone());
                    }
                }
            }
        }
        Ok(val)
    }

    fn finalize(&mut self, reference: &GameRef<Character>) {
        if let Some(liege) = self.liege.clone() {
            if let Ok(mut inner) = liege.try_get_internal_mut() {
                if let Some(liege) = inner.inner_mut() {
                    liege.add_vassal(reference.clone());
                } else {
                    self.liege = None;
                }
            }
        }
        for child in self.children.iter() {
            if let Some(child) = child.get_internal_mut().inner_mut() {
                child.register_parent(reference.clone());
            }
        }
        if self.faith.is_none() {
            if let Some(house) = &self.house {
                if let Some(house) = house.get_internal().inner() {
                    self.faith = Some(house.get_faith());
                }
            }
        }
        if self.culture.is_none() {
            if let Some(house) = &self.house {
                if let Some(house) = house.get_internal().inner() {
                    self.culture = Some(house.get_culture());
                }
            }
        }
        for vassal in self.vassals.iter_mut() {
            match vassal {
                Vassal::Character(c) => {
                    if let Some(c) = c.get_internal_mut().inner_mut() {
                        c.set_liege(reference.clone());
                    }
                }
                Vassal::Reference(c) => {
                    if let Some(c) = c.get_internal_mut().as_mut() {
                        if let Some(c) = c.get_internal_mut().inner_mut() {
                            c.set_liege(reference.clone());
                        }
                    }
                }
            }
        }
    }
}

impl GameObjectDerived for Character {
    fn get_name(&self) -> GameString {
        self.name.clone()
    }

    fn get_references<E: From<EntityRef>, C: Extend<E>>(&self, collection: &mut C) {
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

impl TreeNode<Vec<GameRef<Character>>> for Character {
    fn get_children(&self) -> Option<Vec<GameRef<Character>>> {
        if self.children.is_empty() {
            return None;
        }
        Some(self.children.clone())
    }

    fn get_parent(&self) -> Option<Vec<GameRef<Character>>> {
        if self.parents.is_empty() {
            return None;
        }
        Some(self.parents.clone())
    }

    fn get_class(&self) -> Option<GameString> {
        if let Some(house) = &self.house {
            if let Some(house) = house.get_internal().inner() {
                return Some(house.get_name());
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl ProceduralPath for Character {
    fn get_subdir() -> &'static str {
        "characters"
    }
}

impl Renderable for GameObjectEntity<Character> {
    fn get_template() -> &'static str {
        C_TEMPLATE_NAME
    }
}

impl Localizable for Character {
    fn localize(&mut self, localization: &GameData) -> Result<(), LocalizationError> {
        self.name = localization.localize(&self.name)?;
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
                    } else if stack[1].0 == "GetHerHim" {
                        if self.female {
                            return Some("her".into());
                        } else {
                            return Some("him".into());
                        }
                    } else if stack[1].0 == "GetHerselfHimself" {
                        if self.female {
                            return Some("herself".into());
                        } else {
                            return Some("himself".into());
                        }
                    } else if stack[1].0 == "GetSheHe" {
                        if self.female {
                            return Some("she".into());
                        } else {
                            return Some("he".into());
                        }
                    } else if stack[0].0 == "TARGET_CHARACTER" && stack[1].0 == "GetUIName" {
                        return Some("an unknown assailant".into()); // TODO
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
