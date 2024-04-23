mod renderer;

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

pub trait GameObjectDerived{
    fn from_game_object(base:&'_ GameObject, game_state:&GameState) -> Self;

    fn type_name() -> &'static str;
}
