use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use crate::structures::{Character, Culture, Dynasty, Faith, GameObjectDerived, Memory, Player, Shared, Title};
use crate::game_object::GameObject;

/// A struct representing all known game objects
/// 
/// It is guaranteed to always return a reference to the same object for the same key.
/// Naturally the value of that reference may change as values are added to the game state.
pub struct GameState{
    players: HashMap<String, Shared<Player>>,
    characters: HashMap<String, Shared<Character>>,
    titles: HashMap<String, Shared<Title>>,
    faiths: HashMap<String, Shared<Faith>>,
    cultures: HashMap<String, Shared<Culture>>,
    dynasties: HashMap<String, Shared<Dynasty>>,
    memories: HashMap<String, Shared<Memory>>
}

impl GameState{
    /// Create a new GameState
    pub fn new() -> GameState{
        GameState{
            players: HashMap::new(),
            characters: HashMap::new(),
            titles: HashMap::new(),
            faiths: HashMap::new(),
            cultures: HashMap::new(),
            dynasties: HashMap::new(),
            memories: HashMap::new()
        }
    }

    /// Get a player by key
    pub fn get_player(&mut self, key: &str) -> Shared<Player>{
        if !self.players.contains_key(key){
            let v = Rc::new(RefCell::new(Player::dummy()));
            self.players.insert(key.to_string(), v.clone());
            v
        }
        else{
            self.players.get(key).unwrap().clone()
        }
    }

    /// Get a character by key
    pub fn get_character(&mut self, key: &str) -> Shared<Character>{
        if !self.characters.contains_key(key){
            let v = Rc::new(RefCell::new(Character::dummy()));
            self.characters.insert(key.to_string(), v.clone());
            v
        }
        else{
            self.characters.get(key).unwrap().clone()
        }
    }

    /// Get a title by key
    pub fn get_title(&mut self, key: &str) -> Shared<Title>{
        if !self.titles.contains_key(key){
            let v = Rc::new(RefCell::new(Title::dummy()));
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
            let v = Rc::new(RefCell::new(Faith::dummy()));
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
            let v = Rc::new(RefCell::new(Culture::dummy()));
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
            let v = Rc::new(RefCell::new(Dynasty::dummy()));
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
            let v = Rc::new(RefCell::new(Memory::dummy()));
            self.memories.insert(key.to_string(), v.clone());
            v
        }
    }

    /// Add a player to the game state
    pub fn add_player(&mut self, value: Ref<'_, GameObject>){
        let key = value.get_name().to_string();
        if self.players.contains_key(&key){
            let p = self.players.get(&key).unwrap().clone();
            p.borrow_mut().init(value, self);
        }
        else{
            let p = Player::from_game_object(value, self);
            self.players.insert(key.clone(), Rc::from(RefCell::from(p)));
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
