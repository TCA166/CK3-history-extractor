use derive_more::From;

use super::super::{
    structures::{Character, Culture, Dynasty, Faith, Title},
    types::Shared,
};

/// An enum representing the different types of [super::Renderable] objects
#[derive(From)]
pub enum RenderableType {
    Character(Shared<Character>),
    Culture(Shared<Culture>),
    Dynasty(Shared<Dynasty>),
    Faith(Shared<Faith>),
    Title(Shared<Title>),
}
