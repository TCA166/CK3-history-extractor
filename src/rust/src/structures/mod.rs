mod renderer;

use std::{any::Any, collections::HashMap};

use super::game_object::GameObject;

mod player;
pub use player::Player;

mod character;
pub use character::Character;

mod faith;
pub use faith::Faith;

mod culture;
pub use culture::Culture;

mod dynasty;
pub use dynasty::Dynasty;

mod memory;
pub use memory::Memory;

mod title;
pub use title::Title;

pub trait GameObjectDerived{
    fn from_game_object(base:&'_ GameObject, game_state:&HashMap<String, HashMap<String, GameObjectDerivedType>>) -> Self;
}

pub enum GameObjectDerivedType{
    Player(Player),
    Character(Character),
    Faith(Faith),
    Culture(Culture),
    Dynasty(Dynasty),
    Memory(Memory),
    Title(Title)
}

impl GameObjectDerivedType {
    pub fn get_as<T> (&self) -> Option<&T> where T: GameObjectDerived {
        match self {
            GameObjectDerivedType::Player(player) if player.type_id() == std::any::TypeId::of::<T>() => Some(unsafe { std::mem::transmute::<&Player, &T>(player) }),
            GameObjectDerivedType::Character(character) if character.type_id() == std::any::TypeId::of::<T>() => Some(unsafe { std::mem::transmute::<&Character, &T>(character) }),
            GameObjectDerivedType::Faith(faith) if faith.type_id() == std::any::TypeId::of::<T>() => Some(unsafe { std::mem::transmute::<&Faith, &T>(faith) }),
            GameObjectDerivedType::Culture(culture) if culture.type_id() == std::any::TypeId::of::<T>() => Some(unsafe { std::mem::transmute::<&Culture, &T>(culture) }),
            GameObjectDerivedType::Dynasty(dynasty) if dynasty.type_id() == std::any::TypeId::of::<T>() => Some(unsafe { std::mem::transmute::<&Dynasty, &T>(dynasty) }),
            GameObjectDerivedType::Memory(memory) if memory.type_id() == std::any::TypeId::of::<T>() => Some(unsafe { std::mem::transmute::<&Memory, &T>(memory) }),
            GameObjectDerivedType::Title(title) if title.type_id() == std::any::TypeId::of::<T>() => Some(unsafe { std::mem::transmute::<&Title, &T>(title) }),
            _ => None,
        }
    }
}

