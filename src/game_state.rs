use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use crate::structures::{Character, Culture, DerivedRef, Dynasty, Faith, GameObjectDerived, Memory, Shared, Title};
use crate::game_object::GameObject;

/// A struct representing all known game objects.
/// It is guaranteed to always return a reference to the same object for the same key.
/// Naturally the value of that reference may change as values are added to the game state.
/// This is mainly used during the process of gathering data from the parsed save file.
pub struct GameState{
    /// A character id->Character transform
    characters: HashMap<String, Shared<Character>>,
    /// A title id->Title transform
    titles: HashMap<String, Shared<Title>>,
    /// A faith id->Title transform
    faiths: HashMap<String, Shared<Faith>>,
    /// A culture id->Culture transform
    cultures: HashMap<String, Shared<Culture>>,
    /// A dynasty id->Dynasty transform
    dynasties: HashMap<String, Shared<Dynasty>>,
    /// A memory id->Memory transform
    memories: HashMap<String, Shared<Memory>>,
    /// A trait id->Trait identifier transform
    traits_lookup: Vec<Shared<String>>,
    /// A vassal contract id->Character transform
    contract_transform: HashMap<String, Shared<DerivedRef<Character>>>
}

impl GameState{
    /// Create a new GameState
    pub fn new() -> GameState{
        GameState{
            characters: HashMap::new(),
            titles: HashMap::new(),
            faiths: HashMap::new(),
            cultures: HashMap::new(),
            dynasties: HashMap::new(),
            memories: HashMap::new(),
            traits_lookup: Vec::new(),
            contract_transform: HashMap::new()
        }
    }

    /// Add a lookup table for traits
    pub fn add_lookup(&mut self, array:Vec<Shared<String>>){
        self.traits_lookup = array;
    }

    /// Get a trait by id
    pub fn get_trait(&self, id:u32) -> Shared<String>{
        self.traits_lookup[id as usize].clone()
    }

    /// Get a character by key
    pub fn get_character(&mut self, key: &str) -> Shared<Character>{
        if !self.characters.contains_key(key){
            let v = Rc::new(RefCell::new(Character::dummy(key.parse::<u32>().unwrap())));
            self.characters.insert(key.to_string(), v.clone());
            v
        }
        else{
            self.characters.get(key).unwrap().clone()
        }
    }

    /// Gets the vassal associated with the contract with the given id
    pub fn get_vassal(&mut self, contract_id: &str) -> Shared<DerivedRef<Character>>{
        if !self.contract_transform.contains_key(contract_id){
            let v = Rc::new(RefCell::new(DerivedRef::dummy()));
            self.contract_transform.insert(contract_id.to_string(), v.clone());
            v
        }
        else{
            self.contract_transform.get(contract_id).unwrap().clone()
        }
    }

    /// Adds a new vassal contract
    pub fn add_contract(&mut self, contract_id: &str, character_id: Ref<'_, String>) {
        let char = self.get_character(character_id.as_str());
        if self.contract_transform.contains_key(contract_id){
            let entry = self.contract_transform.get(contract_id).unwrap();
            entry.borrow_mut().init(char);
        }
        else{
            let r = Rc::new(RefCell::new(DerivedRef::from_derived(char)));
            self.contract_transform.insert(contract_id.to_string(), r);
        }
    }

    /// Get a title by key
    pub fn get_title(&mut self, key: &str) -> Shared<Title>{
        if !self.titles.contains_key(key){
            let v = Rc::new(RefCell::new(Title::dummy(key.parse::<u32>().unwrap())));
            self.titles.insert(key.to_string(), v.clone());
            v
        }
        else{
            self.titles.get(key).unwrap().clone()
        }
    }

    /// Get a faith by key
    pub fn get_faith(&mut self, key: &str) -> Shared<Faith>{
        if self.faiths.contains_key(key){
            self.faiths.get(key).unwrap().clone()
        }
        else{
            let v = Rc::new(RefCell::new(Faith::dummy(key.parse::<u32>().unwrap())));
            self.faiths.insert(key.to_string(), v.clone());
            v
        }
    }

    /// Get a culture by key
    pub fn get_culture(&mut self, key: &str) -> Shared<Culture>{
        if self.cultures.contains_key(key){
            self.cultures.get(key).unwrap().clone()
        }
        else{
            let v = Rc::new(RefCell::new(Culture::dummy(key.parse::<u32>().unwrap())));
            self.cultures.insert(key.to_string(), v.clone());
            v
        }
    }

    /// Get a dynasty by key
    pub fn get_dynasty(&mut self, key: &str) -> Shared<Dynasty>{
        if self.dynasties.contains_key(key){
            self.dynasties.get(key).unwrap().clone()
        }
        else{
            let v = Rc::new(RefCell::new(Dynasty::dummy(key.parse::<u32>().unwrap())));
            self.dynasties.insert(key.to_string(), v.clone());
            v
        }
    }

    /// Get a memory by key
    pub fn get_memory(&mut self, key: &str) -> Shared<Memory>{
        if self.memories.contains_key(key){
            self.memories.get(key).unwrap().clone()
        }
        else{
            let v = Rc::new(RefCell::new(Memory::dummy(key.parse::<u32>().unwrap())));
            self.memories.insert(key.to_string(), v.clone());
            v
        }
    }

    /// Add a character to the game state    
    pub fn add_character(&mut self, value: Ref<'_, GameObject>){
        let key = value.get_name().to_string();
        if self.characters.contains_key(&key){
            let c = self.characters.get(&key).unwrap().clone();
            c.borrow_mut().init(value, self);
        }
        else{
            let c = Character::from_game_object(value, self);
            self.characters.insert(key.clone(), Rc::from(RefCell::from(c)));
        }
    }

    /// Add a title to the game state
    pub fn add_title(&mut self, value: Ref<'_, GameObject>){
        let key = value.get_name().to_string();
        if self.titles.contains_key(&key){
            let t = self.titles.get(&key).unwrap().clone();
            t.borrow_mut().init(value, self);
        }
        else{
            let t = Title::from_game_object(value, self);
            self.titles.insert(key.clone(), Rc::from(RefCell::from(t)));
        }
    }

    /// Add a faith to the game state
    pub fn add_faith(&mut self, value: Ref<'_, GameObject>){
        let key = value.get_name().to_string();
        if self.faiths.contains_key(&key){
            let f = self.faiths.get(&key).unwrap().clone();
            f.borrow_mut().init(value, self);
        }
        else{
            let f = Faith::from_game_object(value, self);
            self.faiths.insert(key.clone(), Rc::from(RefCell::from(f)));
        }
    }

    /// Add a culture to the game state
    pub fn add_culture(&mut self, value: Ref<'_, GameObject>){
        let key = value.get_name().to_string();
        if self.cultures.contains_key(&key){
            let c = self.cultures.get(&key).unwrap().clone();
            c.borrow_mut().init(value, self);
        }
        else{
            let c = Culture::from_game_object(value, self);
            self.cultures.insert(key.clone(), Rc::from(RefCell::from(c)));
        }
    }

    /// Add a dynasty to the game state
    pub fn add_dynasty(&mut self, value: Ref<'_, GameObject>){
        let key = value.get_name().to_string();
        if self.dynasties.contains_key(&key){
            let d = self.dynasties.get(key.as_str()).unwrap().clone();
            d.borrow_mut().init(value, self);
        }
        else{
            let d = Dynasty::from_game_object(value, self);
            self.dynasties.insert(key.clone(), Rc::from(RefCell::from(d)));
        }
    }

    /// Add a memory to the game state
    pub fn add_memory(&mut self, value: Ref<'_, GameObject>){
        let key = value.get_name().to_string();
        if self.memories.contains_key(&key){
            let m = self.memories.get(key.as_str()).unwrap().clone();
            m.borrow_mut().init(value, self);
        }
        else{
            let m = Memory::from_game_object(value, self);
            self.memories.insert(key.clone(), Rc::from(RefCell::from(m)));
        }
    }

}
