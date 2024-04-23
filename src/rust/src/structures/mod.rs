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

pub type Shared<T> = std::rc::Rc<std::cell::RefCell<T>>;

pub trait GameObjectDerived{
    fn from_game_object(base:Ref<'_, GameObject>, game_state:&GameState) -> Self;

    fn type_name() -> &'static str;
}
