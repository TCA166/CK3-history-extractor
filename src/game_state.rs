use std::collections::{hash_map::Iter, HashMap};

use super::game_object::{GameId, GameObject, GameString};
use super::structures::{
    Artifact, Character, Culture, DerivedRef, DummyInit, Dynasty, Faith, GameObjectDerived, Memory,
    Title,
};
use super::types::{Shared, Wrapper, WrapperMut};

use serde::{ser::SerializeStruct, Serialize};

/// A struct representing all known game objects.
/// It is guaranteed to always return a reference to the same object for the same key.
/// Naturally the value of that reference may change as values are added to the game state.
/// This is mainly used during the process of gathering data from the parsed save file.
pub struct GameState {
    /// A character id->Character transform
    characters: HashMap<GameId, Shared<Character>>,
    /// A title id->Title transform
    titles: HashMap<GameId, Shared<Title>>,
    /// A faith id->Title transform
    faiths: HashMap<GameId, Shared<Faith>>,
    /// A culture id->Culture transform
    cultures: HashMap<GameId, Shared<Culture>>,
    /// A dynasty id->Dynasty transform
    dynasties: HashMap<GameId, Shared<Dynasty>>,
    /// A memory id->Memory transform
    memories: HashMap<GameId, Shared<Memory>>,
    /// A artifact id->Artifact transform
    artifacts: HashMap<GameId, Shared<Artifact>>,
    /// A trait id->Trait identifier transform
    traits_lookup: Vec<GameString>,
    /// A vassal contract id->Character transform
    contract_transform: HashMap<GameId, Shared<DerivedRef<Character>>>,
}

impl GameState {
    /// Create a new GameState
    pub fn new() -> GameState {
        GameState {
            characters: HashMap::new(),
            titles: HashMap::new(),
            faiths: HashMap::new(),
            cultures: HashMap::new(),
            dynasties: HashMap::new(),
            memories: HashMap::new(),
            artifacts: HashMap::new(),
            traits_lookup: Vec::new(),
            contract_transform: HashMap::new(),
        }
    }

    /// Add a lookup table for traits
    pub fn add_lookup(&mut self, array: Vec<GameString>) {
        self.traits_lookup = array;
    }

    /// Get a trait by id
    pub fn get_trait(&self, id: u16) -> GameString {
        self.traits_lookup[id as usize].clone()
    }

    /// Get a character by key
    pub fn get_character(&mut self, key: &GameId) -> Shared<Character> {
        if !self.characters.contains_key(key) {
            let v = Shared::wrap(Character::dummy(*key));
            self.characters.insert(*key, v.clone());
            v
        } else {
            self.characters.get(key).unwrap().clone()
        }
    }

    /// Gets the vassal associated with the contract with the given id
    pub fn get_vassal(&mut self, contract_id: &GameId) -> Shared<DerivedRef<Character>> {
        if !self.contract_transform.contains_key(contract_id) {
            let v = Shared::wrap(DerivedRef::dummy());
            self.contract_transform.insert(*contract_id, v.clone());
            v
        } else {
            self.contract_transform.get(contract_id).unwrap().clone()
        }
    }

    /// Adds a new vassal contract
    pub fn add_contract(&mut self, contract_id: &GameId, character_id: &GameId) {
        let char = self.get_character(character_id);
        if self.contract_transform.contains_key(contract_id) {
            let entry = self.contract_transform.get(contract_id).unwrap();
            entry.get_internal_mut().init(char);
        } else {
            let r = Shared::wrap(DerivedRef::from_derived(char));
            self.contract_transform.insert(*contract_id, r);
        }
    }

    /// Get a title by key
    pub fn get_title(&mut self, key: &GameId) -> Shared<Title> {
        if !self.titles.contains_key(key) {
            let v = Shared::wrap(Title::dummy(*key));
            self.titles.insert(*key, v.clone());
            v
        } else {
            self.titles.get(key).unwrap().clone()
        }
    }

    /// Get a faith by key
    pub fn get_faith(&mut self, key: &GameId) -> Shared<Faith> {
        if self.faiths.contains_key(key) {
            self.faiths.get(key).unwrap().clone()
        } else {
            let v = Shared::wrap(Faith::dummy(*key));
            self.faiths.insert(*key, v.clone());
            v
        }
    }

    /// Get a culture by key
    pub fn get_culture(&mut self, key: &GameId) -> Shared<Culture> {
        if self.cultures.contains_key(key) {
            self.cultures.get(key).unwrap().clone()
        } else {
            let v = Shared::wrap(Culture::dummy(*key));
            self.cultures.insert(*key, v.clone());
            v
        }
    }

    /// Get a dynasty by key
    pub fn get_dynasty(&mut self, key: &GameId) -> Shared<Dynasty> {
        if self.dynasties.contains_key(key) {
            self.dynasties.get(key).unwrap().clone()
        } else {
            let v = Shared::wrap(Dynasty::dummy(*key));
            self.dynasties.insert(*key, v.clone());
            v
        }
    }

    /// Get a memory by key
    pub fn get_memory(&mut self, key: &GameId) -> Shared<Memory> {
        if self.memories.contains_key(key) {
            self.memories.get(key).unwrap().clone()
        } else {
            let v = Shared::wrap(Memory::dummy(*key));
            self.memories.insert(*key, v.clone());
            v
        }
    }

    /// Get an artifact by key
    pub fn get_artifact(&mut self, key: &GameId) -> Shared<Artifact> {
        if self.artifacts.contains_key(key) {
            self.artifacts.get(key).unwrap().clone()
        } else {
            let v = Shared::wrap(Artifact::dummy(*key));
            self.artifacts.insert(*key, v.clone());
            v
        }
    }

    pub fn add_artifact(&mut self, value: &GameObject) {
        let key = value.get_name().parse::<GameId>().unwrap();
        if self.artifacts.contains_key(&key) {
            let a = self.artifacts.get(&key).unwrap().clone();
            a.get_internal_mut().init(value, self);
        } else {
            let a = Shared::wrap(Artifact::dummy(key));
            self.artifacts.insert(key, a.clone());
            a.get_internal_mut().init(value, self);
        }
    }

    /// Add a character to the game state    
    pub fn add_character(&mut self, value: &GameObject) {
        let key = value.get_name().parse::<GameId>().unwrap();
        if self.characters.contains_key(&key) {
            let c = self.characters.get(&key).unwrap().clone();
            c.get_internal_mut().init(value, self);
        } else {
            let c = Shared::wrap(Character::dummy(key));
            self.characters.insert(key, c.clone());
            c.get_internal_mut().init(value, self);
        }
    }

    /// Add a title to the game state
    pub fn add_title(&mut self, value: &GameObject) {
        let key = value.get_name().parse::<GameId>().unwrap();
        if self.titles.contains_key(&key) {
            let t = self.titles.get(&key).unwrap().clone();
            t.get_internal_mut().init(value, self);
        } else {
            let t = Shared::wrap(Title::dummy(key));
            self.titles.insert(key, t.clone());
            t.get_internal_mut().init(value, self);
        }
    }

    /// Add a faith to the game state
    pub fn add_faith(&mut self, value: &GameObject) {
        let key = value.get_name().parse::<GameId>().unwrap();
        if self.faiths.contains_key(&key) {
            let f = self.faiths.get(&key).unwrap().clone();
            f.get_internal_mut().init(value, self);
        } else {
            let f = Shared::wrap(Faith::dummy(key));
            self.faiths.insert(key, f.clone());
            f.get_internal_mut().init(value, self);
        }
    }

    /// Add a culture to the game state
    pub fn add_culture(&mut self, value: &GameObject) {
        let key = value.get_name().parse::<GameId>().unwrap();
        if self.cultures.contains_key(&key) {
            let c = self.cultures.get(&key).unwrap().clone();
            c.get_internal_mut().init(value, self);
        } else {
            let c = Shared::wrap(Culture::dummy(key));
            self.cultures.insert(key, c.clone());
            c.get_internal_mut().init(value, self);
        }
    }

    /// Add a dynasty to the game state
    pub fn add_dynasty(&mut self, value: &GameObject) {
        let key = value.get_name().parse::<GameId>().unwrap();
        if self.dynasties.contains_key(&key) {
            let d = self.dynasties.get(&key).unwrap().clone();
            d.get_internal_mut().init(value, self);
        } else {
            let d = Shared::wrap(Dynasty::dummy(key));
            self.dynasties.insert(key, d.clone());
            d.get_internal_mut().init(value, self);
        }
    }

    /// Add a memory to the game state
    pub fn add_memory(&mut self, value: &GameObject) {
        let key = value.get_name().parse::<GameId>().unwrap();
        if self.memories.contains_key(&key) {
            let m = self.memories.get(&key).unwrap().clone();
            m.get_internal_mut().init(value, self);
        } else {
            let m = Shared::wrap(Memory::dummy(key));
            self.memories.insert(key, m.clone());
            m.get_internal_mut().init(value, self);
        }
    }

    /// Gets data year->number of deaths for each culture
    pub fn get_culture_graph_data(&self) -> HashMap<GameId, Vec<(u32, u32)>> {
        let mut cultures = HashMap::new();
        for (_, character) in &self.characters {
            let char = character.get_internal();
            let c = char.get_culture();
            if c.is_none() {
                continue;
            }
            let culture = c.unwrap();
            let death_date = char.get_death_date();
            if death_date.is_none() {
                continue;
            }
            let death_date = death_date.unwrap();
            let death_year: u32 = death_date.split_once('.').unwrap().0.parse().unwrap();
            let culture_id = culture.get_internal().get_id();
            let entry = cultures.entry(culture_id).or_insert(HashMap::new());
            let count = entry.entry(death_year).or_insert(0);
            *count += 1;
        }
        // convert the internal hashmaps to vectors
        let mut res = HashMap::new();
        for (culture_id, data) in cultures {
            let mut v = Vec::new();
            for (year, count) in &data {
                v.push((*year, *count));
            }
            let max_yr = data.keys().max().unwrap();
            for yr in 0..=*max_yr {
                if !data.contains_key(&yr)
                    && ((yr != 0 && data.contains_key(&(yr - 1))) || data.contains_key(&(yr + 1)))
                {
                    v.push((yr, 0));
                }
            }
            v.sort_by(|a, b| a.0.cmp(&b.0));
            res.insert(culture_id, v);
        }
        return res;
    }

    /// Returns a hashmap year->number of deaths for a given faith
    pub fn get_faiths_graph_data(&self) -> HashMap<GameId, Vec<(u32, u32)>> {
        let mut faiths = HashMap::new();
        for (_, character) in &self.characters {
            let char = character.get_internal();
            let f = char.get_faith();
            if f.is_none() {
                continue;
            }
            let faith = f.unwrap();
            let death_date = char.get_death_date();
            if death_date.is_none() {
                continue;
            }
            let death_date = death_date.unwrap();
            let death_year: u32 = death_date.split_once('.').unwrap().0.parse().unwrap();
            let faith_id = faith.get_internal().get_id();
            let entry = faiths.entry(faith_id).or_insert(HashMap::new());
            let count = entry.entry(death_year).or_insert(0);
            *count += 1;
        }
        // convert the internal hashmaps to vectors
        let mut res = HashMap::new();
        for (faith_id, data) in faiths {
            let mut v = Vec::new();
            for (year, count) in &data {
                v.push((*year, *count));
            }
            let max_yr = data.keys().max().unwrap();
            for yr in 0..=*max_yr {
                if !data.contains_key(&yr)
                    && ((yr != 0 && data.contains_key(&(yr - 1))) || data.contains_key(&(yr + 1)))
                {
                    v.push((yr, 0));
                }
            }
            v.sort_by(|a, b| a.0.cmp(&b.0));
            res.insert(faith_id, v);
        }
        return res;
    }

    /// Returns a hashmap year->number of deaths for a given dynasty
    pub fn get_title_iter(&self) -> Iter<GameId, Shared<Title>> {
        self.titles.iter()
    }
}

impl Serialize for GameState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("GameState", 7)?;
        state.serialize_field("characters", &self.characters)?;
        state.serialize_field("titles", &self.titles)?;
        state.serialize_field("faiths", &self.faiths)?;
        state.serialize_field("cultures", &self.cultures)?;
        state.serialize_field("dynasties", &self.dynasties)?;
        state.serialize_field("memories", &self.memories)?;
        state.serialize_field("artifacts", &self.artifacts)?;
        state.end()
    }
}
