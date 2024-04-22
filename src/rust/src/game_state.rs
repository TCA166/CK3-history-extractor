use std::collections::HashMap;

use syn::token::Dyn;

use crate::structures::{GameObjectDerived, Player, Character, Title, Faith, Culture, Dynasty, Memory};
use crate::game_object::GameObject;

pub struct GameState{
    players: HashMap<String, Player>,
    characters: HashMap<String, Character>,
    titles: HashMap<String, Title>,
    faiths: HashMap<String, Faith>,
    cultures: HashMap<String, Culture>,
    dynasties: HashMap<String, Dynasty>,
    memories: HashMap<String, Memory>
}

impl GameState{
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

    pub fn get_player(&self, key: &str) -> Option<&Player>{
        self.players.get(key)
    }

    pub fn get_character(&self, key: &str) -> Option<&Character>{
        self.characters.get(key)
    }

    pub fn get_title(&self, key: &str) -> Option<&Title>{
        self.titles.get(key)
    }

    pub fn get_faith(&self, key: &str) -> Option<&Faith>{
        self.faiths.get(key)
    }

    pub fn get_culture(&self, key: &str) -> Option<&Culture>{
        self.cultures.get(key)
    }

    pub fn get_dynasty(&self, key: &str) -> Option<&Dynasty>{
        self.dynasties.get(key)
    }

    pub fn get_memory(&self, key: &str) -> Option<&Memory>{
        self.memories.get(key)
    }

    pub fn add<T>(&mut self, key: String, value: GameObject) where T: GameObjectDerived{
        match T::type_name(){
            "player" => self.players.insert(key, value.get_as::<Player>(&key, self)),
            "character" => self.players.insert(key, value.get_as::<Character>(&key, self)),
            "title" => self.players.insert(key, value.get_as::<Title>(&key, self)),
            "faith" => self.players.insert(key, value.get_as::<Faith>(&key, self)),
            "culture" => self.players.insert(key, value.get_as::<Culture>(&key, self)),
            "dynasty" => self.players.insert(key, value.get_as::<Dynasty>(&key, self)),
            "memory" => self.players.insert(key, value.get_as::<Memory>(&key, self)),
            _ => None
        };
    }


}
