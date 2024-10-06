use serde::Serialize;

use super::{
    parser::{GameId, GameObjectMap, GameState, GameString},
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

/// A submodule that provides an object that can be used on the frontend as a shallow reference to another [GameObjectDerived] object.
mod derived_ref;
pub use derived_ref::{serialize_array, DerivedRef};

/// A submodule that provides the [Artifact] object.
mod artifact;
pub use artifact::Artifact;

/// A trait for objects that can be created from a [GameObjectMap].
/// Currently these include: [Character], [Culture], [Dynasty], [Faith], [Memory], [Player], [Title].
/// The idea is to have uniform interface for the object initialization.
pub trait GameObjectDerived: Serialize {
    /// Get the id of the object.
    /// All CK3 objects have an id that is a number.
    /// Within a given section that number is unique.
    /// For example, all characters have a unique id, but a title and a character can have the same id.
    fn get_id(&self) -> GameId;

    /// Get the name of the object.
    fn get_name(&self) -> GameString;
}

/// A trait for [GameObjectDerived] objects that can be created as a dummy object, only later to be initialized.
pub trait DummyInit: GameObjectDerived {
    /// Create a dummy object that can be used as a placeholder
    /// Can be used to initialize an object from a section yet to be parsed.
    fn dummy(id: GameId) -> Self;

    /// Initialize the object (ideally dummy) with auxiliary data from the game state.
    /// This can be called multiple times, but why would you do that?
    fn init(&mut self, base: &GameObjectMap, game_state: &mut GameState);
}

/// A trait for [GameObjectDerived] objects that can be created from a [GameObjectMap].
pub trait FromGameObject: GameObjectDerived {
    /// Create a new object from a [GameObjectMap].
    fn from_game_object(base: &GameObjectMap, game_state: &mut GameState) -> Self;
}
