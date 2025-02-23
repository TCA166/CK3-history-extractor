use std::{
    any::type_name,
    hash::{Hash, Hasher},
};

use derive_more::From;
use serde::Serialize;

use super::{
    parser::{GameId, GameObjectMap, GameState, GameString, ParsingError},
    types::{Shared, Wrapper},
};

/// A submodule that provides the [Player] object.
mod player;
pub use player::Player;

/// A submodule that provides the [Character] object.
mod character;
pub use character::Character;

/// A submodule that provides the [Faith] object.
mod faith;
pub use faith::Faith;

/// A submodule that provides the [Culture] object.
mod culture;
pub use culture::Culture;

/// A submodule that provides the [Dynasty] object.
mod dynasty;
pub use dynasty::Dynasty;

/// A submodule that provides the [Memory] object.
mod memory;
pub use memory::Memory;

/// A submodule that provides the [Title] object.
mod title;
pub use title::Title;

/// A submodule that provides the [LineageNode] object.
mod lineage;
pub use lineage::LineageNode;

/// A submodule that provides the [Artifact] object.
mod artifact;
pub use artifact::Artifact;

/// A trait for objects that can be created from a [GameObjectMap].
/// Currently these include: [Character], [Culture], [Dynasty], [Faith], [Memory], [Player], [Title].
/// The idea is to have uniform interface for the object initialization.
pub trait GameObjectDerived {
    /// Get the id of the object.
    /// All CK3 objects have an id that is a number.
    /// Within a given section that number is unique.
    /// For example, all characters have a unique id, but a title and a character can have the same id.
    fn get_id(&self) -> GameId;

    /// Get the name of the object.
    /// The result of this method depends on the type.
    fn get_name(&self) -> Option<GameString>;

    /// Get the unique identifier of the object.
    /// This is a tuple of the id and the type name. Since id is unique within a section, and each [GameObjectDerived] type should have an associated section, this tuple is unique.
    fn get_unique_identifier(&self) -> (GameId, &'static str) {
        (self.get_id(), type_name::<Self>())
    }

    /// Extends the provided collection with references to other [GameObjectDerived] objects, if any.
    fn get_references<E: From<GameObjectDerivedType>, C: Extend<E>>(&self, collection: &mut C);
}

impl<T: GameObjectDerived> PartialEq for Shared<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get_internal().get_unique_identifier() == other.get_internal().get_unique_identifier()
    }
}

impl<T: GameObjectDerived> Eq for Shared<T> {}

#[derive(From, Debug, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum GameObjectDerivedType {
    Character(Shared<Character>),
    Culture(Shared<Culture>),
    Dynasty(Shared<Dynasty>),
    Faith(Shared<Faith>),
    Title(Shared<Title>),
    Memory(Shared<Memory>),
    Artifact(Shared<Artifact>),
}

impl GameObjectDerived for GameObjectDerivedType {
    fn get_id(&self) -> GameId {
        match self {
            GameObjectDerivedType::Character(c) => c.get_internal().get_id(),
            GameObjectDerivedType::Culture(c) => c.get_internal().get_id(),
            GameObjectDerivedType::Dynasty(d) => d.get_internal().get_id(),
            GameObjectDerivedType::Faith(f) => f.get_internal().get_id(),
            GameObjectDerivedType::Title(t) => t.get_internal().get_id(),
            GameObjectDerivedType::Memory(m) => m.get_internal().get_id(),
            GameObjectDerivedType::Artifact(a) => a.get_internal().get_id(),
        }
    }

    fn get_name(&self) -> Option<GameString> {
        match self {
            GameObjectDerivedType::Character(c) => c.get_internal().get_name(),
            GameObjectDerivedType::Culture(c) => c.get_internal().get_name(),
            GameObjectDerivedType::Dynasty(d) => d.get_internal().get_name(),
            GameObjectDerivedType::Faith(f) => f.get_internal().get_name(),
            GameObjectDerivedType::Title(t) => t.get_internal().get_name(),
            GameObjectDerivedType::Memory(m) => m.get_internal().get_name(),
            GameObjectDerivedType::Artifact(a) => a.get_internal().get_name(),
        }
    }

    fn get_references<E: From<GameObjectDerivedType>, C: Extend<E>>(&self, collection: &mut C) {
        match self {
            GameObjectDerivedType::Character(c) => c.get_internal().get_references(collection),
            GameObjectDerivedType::Culture(c) => c.get_internal().get_references(collection),
            GameObjectDerivedType::Dynasty(d) => d.get_internal().get_references(collection),
            GameObjectDerivedType::Faith(f) => f.get_internal().get_references(collection),
            GameObjectDerivedType::Title(t) => t.get_internal().get_references(collection),
            GameObjectDerivedType::Memory(m) => m.get_internal().get_references(collection),
            GameObjectDerivedType::Artifact(a) => a.get_internal().get_references(collection),
        }
    }
}

// Type name and id uniquely identify game entities, thus it follows that this type can be hashed using these two values
impl Hash for GameObjectDerivedType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_unique_identifier().hash(state);
    }
}

/// A trait for [GameObjectDerived] objects that can be created as a dummy object, only later to be initialized.
pub trait DummyInit: GameObjectDerived {
    /// Create a dummy object that can be used as a placeholder
    /// Can be used to initialize an object from a section yet to be parsed.
    fn dummy(id: GameId) -> Self;

    /// Initialize the object (ideally dummy) with auxiliary data from the game state.
    /// This can be called multiple times, but why would you do that?
    fn init(
        &mut self,
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<(), ParsingError>;
}

/// A trait for [GameObjectDerived] objects that can be created from a [GameObjectMap].
pub trait FromGameObject: GameObjectDerived + Sized {
    /// Create a new object from a [GameObjectMap].
    fn from_game_object(
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError>;
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_type_eq() {
        assert_ne!(
            GameObjectDerivedType::from(Shared::from(Character::dummy(0))),
            GameObjectDerivedType::from(Shared::from(Title::dummy(0)))
        )
    }

    #[test]
    fn test_set() {
        let mut set = HashSet::new();
        set.insert(GameObjectDerivedType::from(Shared::from(Character::dummy(
            0,
        ))));
        assert!(!set.contains(&GameObjectDerivedType::from(Shared::from(Title::dummy(0)))))
    }
}
