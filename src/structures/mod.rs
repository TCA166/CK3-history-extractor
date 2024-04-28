mod renderer;

use std::cell::Ref;

use super::game_object::GameObject;

use super::game_state::GameState;

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

/// A type alias for shared objects
pub type Shared<T> = std::rc::Rc<std::cell::RefCell<T>>;

/// A trait for objects that can be created from a GameObject
pub trait GameObjectDerived{
    /// Create a new object from a GameObject and auxiliary data from the game state
    fn from_game_object(base:Ref<'_, GameObject>, game_state:&mut GameState) -> Self;

    /// Create a dummy object that can be used as a placeholder
    fn dummy(id:u32) -> Self;

    /// Initialize the object (ideally dummy) with auxiliary data from the game state
    fn init(&mut self, base:Ref<'_, GameObject>, game_state:&mut GameState);

    fn get_id(&self) -> u32;
}
