use serde::Serialize;

use crate::display::{Renderable, GameMap, Grapher, Renderer};
use crate::structures::{Character, Culture, DerivedRef, Dynasty, Faith, GameObjectDerived, Player, Title};

use super::Cullable;

pub enum RenderableType{
    Character(Character),
    Culture(Culture),
    Dynasty(Dynasty),
    Faith(Faith),
    Title(Title),
    Player(Player),
    Ref(DerivedRef<RenderableType>)
}

impl Cullable for RenderableType {
    fn get_depth(&self) -> usize {
        match self{
            RenderableType::Character(c) => c.get_depth(),
            RenderableType::Culture(c) => c.get_depth(),
            RenderableType::Dynasty(d) => d.get_depth(),
            RenderableType::Faith(f) => f.get_depth(),
            RenderableType::Title(t) => t.get_depth(),
            RenderableType::Player(p) => p.get_depth(),
            RenderableType::Ref(r) => r.get_depth()
        }
    }

    fn is_ok(&self) -> bool {
        match self{
            RenderableType::Character(c) => c.is_ok(),
            RenderableType::Culture(c) => c.is_ok(),
            RenderableType::Dynasty(d) => d.is_ok(),
            RenderableType::Faith(f) => f.is_ok(),
            RenderableType::Title(t) => t.is_ok(),
            RenderableType::Player(p) => p.is_ok(),
            RenderableType::Ref(r) => r.is_ok()
        }
    }

    fn set_depth(&mut self, depth:usize, localization:&super::Localizer) {
        match self{
            RenderableType::Character(c) => c.set_depth(depth, localization),
            RenderableType::Culture(c) => c.set_depth(depth, localization),
            RenderableType::Dynasty(d) => d.set_depth(depth, localization),
            RenderableType::Faith(f) => f.set_depth(depth, localization),
            RenderableType::Title(t) => t.set_depth(depth, localization),
            RenderableType::Player(p) => p.set_depth(depth, localization),
            RenderableType::Ref(r) => r.set_depth(depth, localization)
        }
    
    }
}

impl Serialize for RenderableType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self{
            RenderableType::Character(c) => c.serialize(serializer),
            RenderableType::Culture(c) => c.serialize(serializer),
            RenderableType::Dynasty(d) => d.serialize(serializer),
            RenderableType::Faith(f) => f.serialize(serializer),
            RenderableType::Title(t) => t.serialize(serializer),
            RenderableType::Player(p) => p.serialize(serializer),
            RenderableType::Ref(r) => r.serialize(serializer)
        }
    }

}

impl GameObjectDerived for RenderableType {
    fn get_id(&self) -> crate::game_object::GameId {
        match self{
            RenderableType::Character(c) => c.get_id(),
            RenderableType::Culture(c) => c.get_id(),
            RenderableType::Dynasty(d) => d.get_id(),
            RenderableType::Faith(f) => f.get_id(),
            RenderableType::Title(t) => t.get_id(),
            RenderableType::Player(p) => p.get_id(),
            RenderableType::Ref(r) => r.get_id()
        }
    }

    fn get_name(&self) -> crate::game_object::GameString {
        match self{
            RenderableType::Character(c) => c.get_name(),
            RenderableType::Culture(c) => c.get_name(),
            RenderableType::Dynasty(d) => d.get_name(),
            RenderableType::Faith(f) => f.get_name(),
            RenderableType::Title(t) => t.get_name(),
            RenderableType::Player(p) => p.get_name(),
            RenderableType::Ref(r) => r.get_name()
        }
    }
}

impl Renderable for RenderableType{
    fn get_context(&self) -> minijinja::Value {
        match self{
            RenderableType::Character(c) => c.get_context(),
            RenderableType::Culture(c) => c.get_context(),
            RenderableType::Dynasty(d) => d.get_context(),
            RenderableType::Faith(f) => f.get_context(),
            RenderableType::Title(t) => t.get_context(),
            RenderableType::Player(p) => p.get_context(),
            RenderableType::Ref(r) => r.get_context()
        }
    }

    fn get_path(&self, path: &str) -> String {
        match self{
            RenderableType::Character(c) => c.get_path(path),
            RenderableType::Culture(c) => c.get_path(path),
            RenderableType::Dynasty(d) => d.get_path(path),
            RenderableType::Faith(f) => f.get_path(path),
            RenderableType::Title(t) => t.get_path(path),
            RenderableType::Player(p) => p.get_path(path),
            RenderableType::Ref(r) => r.get_path(path)
        }
    }

    fn get_subdir() -> &'static str {
        "."
    }

    fn get_template() -> &'static str {
        "."
    }

    fn render_all(&self, stack:&mut Vec<RenderableType>, renderer: &mut Renderer, game_map: Option<&GameMap>, grapher: Option<&Grapher>) {
        match self{
            RenderableType::Character(c) => c.render_all(stack, renderer, game_map, grapher),
            RenderableType::Culture(c) => c.render_all(stack, renderer, game_map, grapher),
            RenderableType::Dynasty(d) => d.render_all(stack, renderer, game_map, grapher),
            RenderableType::Faith(f) => f.render_all(stack, renderer, game_map, grapher),
            RenderableType::Title(t) => t.render_all(stack, renderer, game_map, grapher),
            RenderableType::Player(p) => p.render_all(stack, renderer, game_map, grapher),
            RenderableType::Ref(r) => r.render_all(stack, renderer, game_map, grapher)
        }
    }

}
