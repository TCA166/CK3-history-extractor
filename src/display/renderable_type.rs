use serde::Serialize;

use super::{
    super::{
        parser::{GameId, GameString},
        structures::{Character, Culture, Dynasty, Faith, GameObjectDerived, Title},
        types::{Shared, Wrapper, WrapperMut},
    },
    Cullable,
};

/// An enum representing the different types of [super::Renderable] objects
pub enum RenderableType {
    Character(Shared<Character>),
    Culture(Shared<Culture>),
    Dynasty(Shared<Dynasty>),
    Faith(Shared<Faith>),
    Title(Shared<Title>),
}

impl Cullable for RenderableType {
    fn get_depth(&self) -> usize {
        match self {
            RenderableType::Character(c) => c.get_internal().get_depth(),
            RenderableType::Culture(c) => c.get_internal().get_depth(),
            RenderableType::Dynasty(d) => d.get_internal().get_depth(),
            RenderableType::Faith(f) => f.get_internal().get_depth(),
            RenderableType::Title(t) => t.get_internal().get_depth(),
        }
    }

    fn is_ok(&self) -> bool {
        match self {
            RenderableType::Character(c) => c.get_internal().is_ok(),
            RenderableType::Culture(c) => c.get_internal().is_ok(),
            RenderableType::Dynasty(d) => d.get_internal().is_ok(),
            RenderableType::Faith(f) => f.get_internal().is_ok(),
            RenderableType::Title(t) => t.get_internal().is_ok(),
        }
    }

    fn set_depth(&mut self, depth: usize) {
        match self {
            RenderableType::Character(c) => c.get_internal_mut().set_depth(depth),
            RenderableType::Culture(c) => c.get_internal_mut().set_depth(depth),
            RenderableType::Dynasty(d) => d.get_internal_mut().set_depth(depth),
            RenderableType::Faith(f) => f.get_internal_mut().set_depth(depth),
            RenderableType::Title(t) => t.get_internal_mut().set_depth(depth),
        }
    }
}

impl Serialize for RenderableType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RenderableType::Character(c) => c.serialize(serializer),
            RenderableType::Culture(c) => c.serialize(serializer),
            RenderableType::Dynasty(d) => d.serialize(serializer),
            RenderableType::Faith(f) => f.serialize(serializer),
            RenderableType::Title(t) => t.serialize(serializer),
        }
    }
}

impl GameObjectDerived for RenderableType {
    fn get_id(&self) -> GameId {
        match self {
            RenderableType::Character(c) => c.get_internal().get_id(),
            RenderableType::Culture(c) => c.get_internal().get_id(),
            RenderableType::Dynasty(d) => d.get_internal().get_id(),
            RenderableType::Faith(f) => f.get_internal().get_id(),
            RenderableType::Title(t) => t.get_internal().get_id(),
        }
    }

    fn get_name(&self) -> GameString {
        match self {
            RenderableType::Character(c) => c.get_internal().get_name(),
            RenderableType::Culture(c) => c.get_internal().get_name(),
            RenderableType::Dynasty(d) => d.get_internal().get_name(),
            RenderableType::Faith(f) => f.get_internal().get_name(),
            RenderableType::Title(t) => t.get_internal().get_name(),
        }
    }
}
