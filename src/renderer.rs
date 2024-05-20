use std::collections::HashMap;

use minijinja::Environment;
use serde::Serialize;

use crate::{game_object::GameId, graph::Grapher, localizer::Localizer, map::GameMap, structures::GameObjectDerived};

/// A struct that renders objects into html pages.
/// It holds a reference to the [Environment] that is used to render the templates, tracks which objects have been rendered and holds the root path.
pub struct Renderer<'a>{
    env: &'a Environment<'a>,
    rendered: HashMap<&'static str, HashMap<GameId, bool>>,
    path: String
}

impl<'a> Renderer<'a>{
    /// Create a new Renderer with the given [Environment] and path.
    pub fn new(env: &'a Environment<'a>, path: String) -> Self{
        Renderer{
            env,
            rendered: HashMap::new(),
            path
        }
    }

    /// Returns true if the object has already been rendered.
    fn is_rendered<T: Renderable>(&self, id: GameId) -> bool{
        let rendered = self.rendered.get(T::get_subdir());
        if rendered.is_none(){
            return false
        }
        let rendered = rendered.unwrap().get(&id);
        if rendered.is_none(){
            return false
        }
        *rendered.unwrap()
    }

    /// Renders the object and returns true if it was rendered.
    pub fn render<T: Renderable + Cullable>(&mut self, obj: &T) -> bool{
        //if it is rendered then return
        if self.is_rendered::<T>(obj.get_id()) || !obj.is_ok(){
            return false
        }
        let ctx = obj.get_context();
        let contents = self.env.get_template(T::get_template()).unwrap().render(&ctx).unwrap();
        let path = obj.get_path(&self.path);
        std::fs::write(path, contents).unwrap();
        let rendered = self.rendered.entry(T::get_subdir()).or_insert(HashMap::new());
        rendered.insert(obj.get_id(), true);
        return true;
    }

    pub fn get_path(&self) -> &str{
        &self.path
    }
}

/// Trait for objects that can be rendered into a html page.
/// Since this uses [minijinja] the [serde::Serialize] trait is also needed.
/// Each object that implements this trait should have a corresponding template file in the templates folder.
pub trait Renderable: Serialize + GameObjectDerived{
    /// Returns the template file name.
    fn get_template() -> &'static str;

    /// Returns the context that will be used to render the object.
    fn get_context(&self) -> minijinja::Value;

    /// Returns the subdirectory name where the object should be written to.
    fn get_subdir() -> &'static str;

    /// Returns the path where the object should be written to.
    fn get_path(&self, path: &str) -> String{
        format!("{}/{}/{}.html", path, Self::get_subdir(), self.get_id())
    }

    /// Renders the object and all the references of the object if they are not already rendered.
    fn render_all(&self, renderer: &mut Renderer, game_map: Option<&GameMap>, grapher: Option<&Grapher>);
}

/// Trait for objects that can be culled.
/// This is used to limit object serialization to a certain depth.
/// Not all [Renderable] objects need to implement this trait.
pub trait Cullable : GameObjectDerived{
    /// Set the depth of the object and performs localization.
    /// Ideally this should be called on the root object once and the depth should be propagated to all children.
    /// Also ideally should do nothing if the depth is less than or equal to the current depth.
    fn set_depth(&mut self, depth:usize, localization:&Localizer);

    /// Get the depth of the object.
    fn get_depth(&self) -> usize;

    /// Returns true if the object is ok to be rendered / serialized.
    fn is_ok(&self) -> bool{
        self.get_depth() > 0
    }
}
