
/// A submodule that provides [Renderable] and [Cullable] traits for objects that can be rendered.
mod renderer;
pub use renderer::{Cullable, Renderer, Renderable};

use std::rc::Rc;
use std::cell::{BorrowMutError, RefCell, RefMut};

use crate::game_object::{GameString, RefOrRaw, WrapperMut};

use super::game_object::{GameObject, GameId, Wrapper};

use super::game_state::GameState;

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
pub use derived_ref::{DerivedRef, serialize_array};

/// A type alias for shared objects.
/// Aliases: [std::rc::GameString]<[std::cell::RefCell]<>>
/// 
/// # Example
/// 
/// ```
/// let obj:Shared<String> = Shared::wrap("Hello");
/// 
/// let value:Ref<String> = obj.get_internal();
/// ```
pub type Shared<T> = Rc<RefCell<T>>;

impl<T> Wrapper<T> for Shared<T> {
    fn wrap(t:T) -> Self {
        Rc::new(RefCell::new(t))
    }

    fn get_internal(&self) -> RefOrRaw<T> {
        RefOrRaw::Ref(self.borrow())
    }

    fn try_get_internal(&self) -> Result<RefOrRaw<T>, std::cell::BorrowError> {
        let r = self.try_borrow();
        match r {
            Ok(r) => Ok(RefOrRaw::Ref(r)),
            Err(e) => Err(e)
        }
    }
}

impl<T> WrapperMut<T> for Shared<T> {
    fn get_internal_mut(&self) -> RefMut<T> {
        self.borrow_mut()
    }

    fn try_get_internal_mut(&self) -> Result<RefMut<T>, BorrowMutError> {
        self.try_borrow_mut()
    }
}

/// A trait for objects that can be created from a [GameObject].
/// Currently these include: [Character], [Culture], [Dynasty], [Faith], [Memory], [Player], [Title].
/// The idea is to have uniform interface for the object initialization.
pub trait GameObjectDerived{
    /// Create a new object from a GameObject and auxiliary data from the game state.
    fn from_game_object(base:&GameObject, game_state:&mut GameState) -> Self;

    /// Create a dummy object that can be used as a placeholder
    /// Can be used to initialize an object from a section yet to be parsed.
    fn dummy(id:GameId) -> Self;

    /// Initialize the object (ideally dummy) with auxiliary data from the game state.
    /// This can be called multiple times, but why would you do that?
    fn init(&mut self, base:&GameObject, game_state:&mut GameState);

    /// Get the id of the object.
    /// All CK3 objects have an id that is a number.
    /// Within a given section that number is unique.
    /// For example, all characters have a unique id, but a title and a character can have the same id.
    fn get_id(&self) -> GameId;

    /// Get the name of the object.
    fn get_name(&self) -> GameString;
}
