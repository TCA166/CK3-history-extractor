use std::{
    any::type_name,
    hash::{Hash, Hasher},
};

use super::{
    parser::{GameId, GameObjectMap, GameState, GameString, ParsingError},
    types::{Shared, Wrapper},
};

/// A submodule that provides the [Player] object.
mod player;
use derive_more::From;
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
use serde::Serialize;
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
    fn new(
        id: GameId,
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError>
    where
        Self: Sized;

    /// Get the name of the object.
    /// The result of this method depends on the type.
    fn get_name(&self) -> GameString;

    /// Extends the provided collection with references to other [GameObjectDerived] objects, if any.
    fn get_references<T: GameObjectDerived, E: From<GameObjectEntity<T>>, C: Extend<E>>(
        &self,
        collection: &mut C,
    );
}

#[derive(Serialize, Debug)]
pub struct GameObjectEntity<T: GameObjectDerived> {
    id: GameId,
    #[serde(flatten)]
    entity: Option<T>,
}

impl<T: GameObjectDerived> GameObjectEntity<T> {
    pub fn new(id: GameId) -> Self {
        Self { id, entity: None }
    }

    pub fn get_id(&self) -> GameId {
        self.id
    }

    /// Get the unique identifier of the object.
    /// This is a tuple of the id and the type name.
    pub fn get_unique_identifier(&self) -> (GameId, &'static str) {
        (self.get_id(), type_name::<T>())
    }

    pub fn init(
        &mut self,
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<(), ParsingError> {
        self.entity = Some(T::new(self.id, base, game_state)?);
        Ok(())
    }

    pub fn inner(&self) -> Option<&T> {
        self.entity.as_ref()
    }
}

pub type GameRef<T> = Shared<GameObjectEntity<T>>;

impl<T: GameObjectDerived> Hash for GameObjectEntity<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_unique_identifier().hash(state);
    }
}

#[derive(From, Debug, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum EntityRef {
    Character(Character),
    Culture(Culture),
    Dynasty(Dynasty),
    Faith(Faith),
    Title(Title),
    Memory(Memory),
    Artifact(Artifact),
}
