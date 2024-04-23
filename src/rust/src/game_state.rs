use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use crate::structures::{Character, Culture, Dynasty, Faith, GameObjectDerived, Memory, Player, Shared, Title};
use crate::game_object::GameObject;

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

    pub fn get_player(&self, key: &str) -> Option<&Shared<Player>>{
        self.players.get(key)
    }

    pub fn get_character(&self, key: &str) -> Option<&Shared<Character>>{
        self.characters.get(key)
    }

    pub fn get_title(&self, key: &str) -> Option<&Shared<Title>>{
        self.titles.get(key)
    }

    pub fn get_faith(&self, key: &str) -> Option<&Shared<Faith>>{
        self.faiths.get(key)
    }

    pub fn get_culture(&self, key: &str) -> Option<&Shared<Culture>>{
        self.cultures.get(key)
    }

    pub fn get_dynasty(&self, key: &str) -> Option<&Shared<Dynasty>>{
        self.dynasties.get(key)
    }

    pub fn get_memory(&self, key: &str) -> Option<&Shared<Memory>>{
        self.memories.get(key)
    }

    pub fn add<T>(&mut self, value: Ref<'_, GameObject>) where T: GameObjectDerived{
        let key = value.get_name().to_string();
        match T::type_name(){
            "player" => {
                self.players.insert(key.clone(), Rc::from(RefCell::from(Player::from_game_object(value, self))));
            },
            "character" => {
                self.characters.insert(key.clone(), Rc::from(RefCell::from(Character::from_game_object(value, self))));
            },
            "title" => {
                self.titles.insert(key.clone(), Rc::from(RefCell::from(Title::from_game_object(value, self))));
            },
            "faith" => {
                self.faiths.insert(key.clone(), Rc::from(RefCell::from(Faith::from_game_object(value, self))));
            },
            "culture" => {
                self.cultures.insert(key.clone(), Rc::from(RefCell::from(Culture::from_game_object(value, self))));
            },
            "dynasty" => {
                self.dynasties.insert(key.clone(), Rc::from(RefCell::from(Dynasty::from_game_object(value, self))));
            },
            "memory" => {
                self.memories.insert(key.clone(), Rc::from(RefCell::from(Memory::from_game_object(value, self))));
            },
            _ => {panic!("Unknown type: {}", T::type_name())}
        };
    }


}
