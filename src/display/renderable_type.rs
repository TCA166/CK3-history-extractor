use serde::Serialize;

use super::{
    super::{
        parser::{GameId, GameString},
        structures::{Character, Culture, Dynasty, Faith, GameObjectDerived, Player, Title},
        types::{Shared, Wrapper, WrapperMut},
    },
    Cullable, Renderable, Renderer,
};

/// An enum representing the different types of [Renderable] objects
pub enum RenderableType<'a> {
    Character(Shared<Character>),
    Culture(Shared<Culture>),
    Dynasty(Shared<Dynasty>),
    Faith(Shared<Faith>),
    Title(Shared<Title>),
    Player(&'a mut Player),
}

impl<'a> Cullable for RenderableType<'a> {
    fn get_depth(&self) -> usize {
        match self {
            RenderableType::Character(c) => c.get_internal().get_depth(),
            RenderableType::Culture(c) => c.get_internal().get_depth(),
            RenderableType::Dynasty(d) => d.get_internal().get_depth(),
            RenderableType::Faith(f) => f.get_internal().get_depth(),
            RenderableType::Title(t) => t.get_internal().get_depth(),
            RenderableType::Player(p) => p.get_depth(),
        }
    }

    fn is_ok(&self) -> bool {
        match self {
            RenderableType::Character(c) => c.get_internal().is_ok(),
            RenderableType::Culture(c) => c.get_internal().is_ok(),
            RenderableType::Dynasty(d) => d.get_internal().is_ok(),
            RenderableType::Faith(f) => f.get_internal().is_ok(),
            RenderableType::Title(t) => t.get_internal().is_ok(),
            RenderableType::Player(p) => p.is_ok(),
        }
    }

    fn set_depth(&mut self, depth: usize) {
        match self {
            RenderableType::Character(c) => c.get_internal_mut().set_depth(depth),
            RenderableType::Culture(c) => c.get_internal_mut().set_depth(depth),
            RenderableType::Dynasty(d) => d.get_internal_mut().set_depth(depth),
            RenderableType::Faith(f) => f.get_internal_mut().set_depth(depth),
            RenderableType::Title(t) => t.get_internal_mut().set_depth(depth),
            RenderableType::Player(p) => p.set_depth(depth),
        }
    }
}

impl<'a> Serialize for RenderableType<'a> {
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
            RenderableType::Player(p) => p.serialize(serializer),
        }
    }
}

impl<'a> GameObjectDerived for RenderableType<'a> {
    fn get_id(&self) -> GameId {
        match self {
            RenderableType::Character(c) => c.get_internal().get_id(),
            RenderableType::Culture(c) => c.get_internal().get_id(),
            RenderableType::Dynasty(d) => d.get_internal().get_id(),
            RenderableType::Faith(f) => f.get_internal().get_id(),
            RenderableType::Title(t) => t.get_internal().get_id(),
            RenderableType::Player(p) => p.get_id(),
        }
    }

    fn get_name(&self) -> GameString {
        match self {
            RenderableType::Character(c) => c.get_internal().get_name(),
            RenderableType::Culture(c) => c.get_internal().get_name(),
            RenderableType::Dynasty(d) => d.get_internal().get_name(),
            RenderableType::Faith(f) => f.get_internal().get_name(),
            RenderableType::Title(t) => t.get_internal().get_name(),
            RenderableType::Player(p) => p.get_name(),
        }
    }
}

// not a full implementation of Renderable because getting the static struct fields is not possible
impl<'a> RenderableType<'a> {
    pub fn render_all(&self, stack: &mut Vec<RenderableType>, renderer: &mut Renderer) {
        match self {
            RenderableType::Character(c) => c.get_internal().render_all(stack, renderer),
            RenderableType::Culture(c) => c.get_internal().render_all(stack, renderer),
            RenderableType::Dynasty(d) => d.get_internal().render_all(stack, renderer),
            RenderableType::Faith(f) => f.get_internal().render_all(stack, renderer),
            RenderableType::Title(t) => t.get_internal().render_all(stack, renderer),
            RenderableType::Player(p) => p.render_all(stack, renderer),
        }
    }
}
