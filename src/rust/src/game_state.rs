use std::collections::HashMap;
use std::rc::Rc;

use crate::structures::{GameObjectDerived, Player, Character, Title, Faith, Culture, Dynasty, Memory};
use crate::game_object::GameObject;

pub struct GameState{
    players: HashMap<String, Rc<Player>>,
    characters: HashMap<String, Rc<Character>>,
    titles: HashMap<String, Rc<Title>>,
    faiths: HashMap<String, Rc<Faith>>,
    cultures: HashMap<String, Rc<Culture>>,
    dynasties: HashMap<String, Rc<Dynasty>>,
    memories: HashMap<String, Rc<Memory>>
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

    pub fn get_player(&self, key: &str) -> Option<&Rc<Player>>{
        self.players.get(key)
    }

    pub fn get_character(&self, key: &str) -> Option<&Rc<Character>>{
        self.characters.get(key)
    }

    pub fn get_title(&self, key: &str) -> Option<&Rc<Title>>{
        self.titles.get(key)
    }

    pub fn get_faith(&self, key: &str) -> Option<&Rc<Faith>>{
        self.faiths.get(key)
    }

    pub fn get_culture(&self, key: &str) -> Option<&Rc<Culture>>{
        self.cultures.get(key)
    }

    pub fn get_dynasty(&self, key: &str) -> Option<&Rc<Dynasty>>{
        self.dynasties.get(key)
    }

    pub fn get_memory(&self, key: &str) -> Option<&Rc<Memory>>{
        self.memories.get(key)
    }

    pub fn add<T>(&mut self, value: &GameObject) where T: GameObjectDerived{
        let key = value.get_name().to_string();
        match T::type_name(){
            "player" => {
                self.players.insert(key.clone(), Rc::from(value.get_as::<Player>(&key, self)));
            },
            "character" => {
                self.characters.insert(key.clone(), Rc::from(value.get_as::<Character>(&key, self)));
            },
            "title" => {
                self.titles.insert(key.clone(), Rc::from(value.get_as::<Title>(&key, self)));
            },
            "faith" => {
                self.faiths.insert(key.clone(), Rc::from(value.get_as::<Faith>(&key, self)));
            },
            "culture" => {
                self.cultures.insert(key.clone(), Rc::from(value.get_as::<Culture>(&key, self)));
            },
            "dynasty" => {
                self.dynasties.insert(key.clone(), Rc::from(value.get_as::<Dynasty>(&key, self)));
            },
            "memory" => {
                self.memories.insert(key.clone(), Rc::from(value.get_as::<Memory>(&key, self)));
            },
            _ => {panic!("Unknown type: {}", T::type_name())}
        };
    }


}
