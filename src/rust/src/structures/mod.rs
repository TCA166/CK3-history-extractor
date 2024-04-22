mod renderer;

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

pub enum FrontendStructure<'a>{
    Character(Character<'a>),
    Faith(Faith<'a>),
    Culture(Culture<'a>),
    Dynasty(Dynasty<'a>),
    Title(Title<'a>),
}
