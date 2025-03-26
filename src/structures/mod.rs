use std::{
    any::type_name,
    hash::{Hash, Hasher},
};

use super::{
    game_data::{Localizable, LocalizationError, Localize},
    parser::{GameObjectMap, GameRef, GameState, ParsingError},
    types::{GameId, GameString, Wrapper, WrapperMut},
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

mod house;
pub use house::House;

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

pub trait FromGameObject: GameObjectDerived {
    fn from_game_object(
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError>;

    #[allow(unused_variables)]
    fn finalize(&mut self, reference: &GameRef<Self>) {}
}

/// A trait for objects that can be created from a [GameObjectMap].
/// Currently these include: [Character], [Culture], [Dynasty], [Faith], [Memory], [Player], [Title].
/// The idea is to have uniform interface for the object initialization.
pub trait GameObjectDerived: Sized {
    /// Get the name of the object.
    /// The result of this method depends on the type.
    fn get_name(&self) -> GameString;

    /// Extends the provided collection with references to other [GameObjectDerived] objects, if any.
    fn get_references<E: From<EntityRef>, C: Extend<E>>(&self, collection: &mut C);
}

#[derive(Serialize, Debug)]
pub struct GameObjectEntity<T: GameObjectDerived> {
    id: GameId,
    /* TODO I would for there to be a way to make this NOT an option,
    naturally this is an option because in the current model we trust structures
    that the IDs are valid, but sometimes they arent, meaning we cant implement
    Deref. If we wanted to fix this we would need to have two sets of
    structures, converting between them in the finalize step but thats a big rework
    */
    #[serde(flatten)]
    entity: Option<T>,
}

impl<T: GameObjectDerived + FromGameObject> GameObjectEntity<T> {
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
        self.entity = Some(T::from_game_object(base, game_state)?);
        Ok(())
    }

    pub fn inner(&self) -> Option<&T> {
        self.entity.as_ref()
    }

    pub fn inner_mut(&mut self) -> Option<&mut T> {
        self.entity.as_mut()
    }

    pub fn replace(&mut self, entity: T) {
        self.entity.replace(entity);
    }
}

impl<T: GameObjectDerived + FromGameObject> PartialEq for GameObjectEntity<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get_unique_identifier() == other.get_unique_identifier()
    }
}

impl<T: GameObjectDerived + FromGameObject> Eq for GameObjectEntity<T> {}

impl<T: GameObjectDerived + FromGameObject> Hash for GameObjectEntity<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_unique_identifier().hash(state);
    }
}

impl<T: Localizable + GameObjectDerived + FromGameObject> Localizable for GameRef<T> {
    fn localize<L: Localize<GameString>>(
        &mut self,
        localization: &mut L,
    ) -> Result<(), LocalizationError> {
        if let Some(entity) = self.get_internal_mut().entity.as_mut() {
            entity.finalize(self);
            entity.localize(localization)
        } else {
            Ok(())
        }
    }
}

impl<T: GameObjectDerived + FromGameObject> PartialEq for GameRef<T> {
    fn eq(&self, other: &Self) -> bool {
        if let Ok(a) = self.try_get_internal() {
            if let Ok(b) = other.try_get_internal() {
                a.get_unique_identifier() == b.get_unique_identifier()
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl<T: GameObjectDerived + FromGameObject> Eq for GameRef<T> {}

impl<T: GameObjectDerived + FromGameObject> Hash for GameRef<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if let Ok(internal) = self.try_get_internal() {
            internal.get_unique_identifier().hash(state);
        }
    }
}

#[derive(From, PartialEq, Eq)]
pub enum EntityRef {
    Character(GameRef<Character>),
    Culture(GameRef<Culture>),
    Dynasty(GameRef<Dynasty>),
    House(GameRef<House>),
    Faith(GameRef<Faith>),
    Title(GameRef<Title>),
    Memory(GameRef<Memory>),
    Artifact(GameRef<Artifact>),
}

impl Hash for EntityRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            EntityRef::Character(char) => char.get_internal().hash(state),
            EntityRef::Culture(cul) => cul.get_internal().hash(state),
            EntityRef::Dynasty(dynasty) => dynasty.get_internal().hash(state),
            EntityRef::House(house) => house.get_internal().hash(state),
            EntityRef::Faith(faith) => faith.get_internal().hash(state),
            EntityRef::Title(title) => title.get_internal().hash(state),
            EntityRef::Memory(mem) => mem.get_internal().hash(state),
            EntityRef::Artifact(art) => art.get_internal().hash(state),
        }
    }
}

impl GameObjectDerived for EntityRef {
    fn get_name(&self) -> GameString {
        match self {
            EntityRef::Character(char) => char.get_internal().inner().unwrap().get_name(),
            EntityRef::Culture(cul) => cul.get_internal().inner().unwrap().get_name(),
            EntityRef::Dynasty(dynasty) => dynasty.get_internal().inner().unwrap().get_name(),
            EntityRef::House(house) => house.get_internal().inner().unwrap().get_name(),
            EntityRef::Faith(faith) => faith.get_internal().inner().unwrap().get_name(),
            EntityRef::Title(title) => title.get_internal().inner().unwrap().get_name(),
            EntityRef::Memory(mem) => mem.get_internal().inner().unwrap().get_name(),
            EntityRef::Artifact(art) => art.get_internal().inner().unwrap().get_name(),
        }
    }

    fn get_references<E: From<EntityRef>, C: Extend<E>>(&self, collection: &mut C) {
        match self {
            EntityRef::Character(char) => char
                .get_internal()
                .inner()
                .map(|v| v.get_references(collection)),
            EntityRef::Culture(cul) => cul
                .get_internal()
                .inner()
                .map(|v| v.get_references(collection)),
            EntityRef::Dynasty(dynasty) => dynasty
                .get_internal()
                .inner()
                .map(|v| v.get_references(collection)),
            EntityRef::House(house) => house
                .get_internal()
                .inner()
                .map(|v| v.get_references(collection)),
            EntityRef::Faith(faith) => faith
                .get_internal()
                .inner()
                .map(|v| v.get_references(collection)),
            EntityRef::Title(title) => title
                .get_internal()
                .inner()
                .map(|v| v.get_references(collection)),
            EntityRef::Memory(mem) => mem
                .get_internal()
                .inner()
                .map(|v| v.get_references(collection)),
            EntityRef::Artifact(art) => art
                .get_internal()
                .inner()
                .map(|v| v.get_references(collection)),
        };
    }
}
