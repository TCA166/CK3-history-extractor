use std::{
    any::type_name,
    hash::{Hash, Hasher},
};

use super::{
    super::game_data::{GameData, Localizable, LocalizationError},
    game_state::GameState,
    parser::{
        types::{GameId, GameString, Shared, Wrapper, WrapperMut},
        GameObjectMap, ParsingError,
    },
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
pub use title::Title;

/// A submodule that provides the [LineageNode] object.
mod lineage;
pub use lineage::LineageNode;

/// A submodule that provides the [Artifact] object.
mod artifact;
pub use artifact::Artifact;

/// A reference to a game object.
pub type GameRef<T> = Shared<GameObjectEntity<T>>;

// TODO I would like for Finalize to consume the object and return a 'finalized' version of it.

/// Needs to be finalized before being used.
/// Meant to to be implemented for different variants of [GameObjectEntity]
///
/// ## Implementing Finalize
///
/// Let's say, that we have a [GameObjectEntity] of type [Character], and we
/// want to have a two way relationship with another entity, maybe even another
/// character. This shouldn't be done during creation, as that other struct
/// might not exist yet in memory. [Finalize::finalize] is the place to do it.
///
/// ## Finalization process
///
/// So generally, finalization is meant to be done after all [GameObjectDerived]
/// have been created. After that, the [Finalize::finalize] method is called
/// on each object, generally by the [super::GameState].
pub trait Finalize {
    /// Resolves all the dangling references and performs any necessary
    /// finalization tasks.
    #[allow(unused_variables)]
    fn finalize(&mut self) {}
}

/// For [GameObjectDerived] structs that are derived from a single [GameObjectMap].
pub trait FromGameObject: GameObjectDerived {
    /// Creates a new instance of the struct from the provided [GameObjectMap].
    fn from_game_object(
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError>;
}

/// A trait for objects that come from a SaveFile.
/// Currently these include: [Character], [Culture], [Dynasty], [Faith], [Memory], [Player], [Title].
pub trait GameObjectDerived: Sized {
    /// Get the name of the object.
    /// The result of this method depends on the type.
    fn get_name(&self) -> GameString;

    /// Extends the provided collection with references to other [GameObjectDerived] objects, if any.
    fn get_references<E: From<EntityRef>, C: Extend<E>>(&self, collection: &mut C);
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GameObjectEntity<T: GameObjectDerived> {
    id: GameId,
    /* TODO I would for there to be a way to make this NOT an option,
    naturally this is an option because in the current model we trust structures
    that the IDs are valid, but sometimes they arent, meaning we cant implement
    Deref. If we wanted to fix this we would need to have two sets of
    structures, converting between them in the finalize step but thats a big rework
    */
    #[cfg_attr(feature = "serde", serde(flatten))]
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
    fn localize(&mut self, localization: &GameData) -> Result<(), LocalizationError> {
        if let Some(entity) = self.get_internal_mut().entity.as_mut() {
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

#[derive(From, PartialEq, Eq, Hash)]
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
