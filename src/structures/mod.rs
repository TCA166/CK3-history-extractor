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

mod lineage;
pub use lineage::LineageNode;

/// A type alias for shared objects.
/// Aliases: [std::rc::Rc]<[std::cell::RefCell]<>>
/// 
/// # Example
/// 
/// ```
/// let obj:Shared<String> = Rc::new(RefCell::new("Hello"));
/// 
/// let value:Ref<String> = obj.borrow();
/// ```
pub type Shared<T> = std::rc::Rc<std::cell::RefCell<T>>;

/// A trait for objects that can be created from a [GameObject].
/// Currently these include: [Character], [Culture], [Dynasty], [Faith], [Memory], [Player], [Title].
/// The idea is to have uniform interface for the object initialization.
pub trait GameObjectDerived{
    /// Create a new object from a GameObject and auxiliary data from the game state.
    fn from_game_object(base:Ref<'_, GameObject>, game_state:&mut GameState) -> Self;

    /// Create a dummy object that can be used as a placeholder
    /// Can be used to initialize an object from a section yet to be parsed.
    fn dummy(id:u32) -> Self;

    /// Initialize the object (ideally dummy) with auxiliary data from the game state.
    /// This can be called multiple times, but why would you do that?
    fn init(&mut self, base:Ref<'_, GameObject>, game_state:&mut GameState);

    /// Get the id of the object.
    /// All CK3 objects have an id that is a number.
    /// Within a given section that number is unique.
    /// For example, all characters have a unique id, but a title and a character can have the same id.
    fn get_id(&self) -> u32;
}
