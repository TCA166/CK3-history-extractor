use std::{
    collections::{HashMap, HashSet},
    fs, thread,
};

use minijinja::{Environment, Value};

use serde::Serialize;

use super::super::{game_object::GameId, game_state::GameState, structures::GameObjectDerived};
use super::{graph::Grapher, localizer::Localizer, map::GameMap, RenderableType};

/// A struct that renders objects into html pages.
/// It holds a reference to the [Environment] that is used to render the templates, tracks which objects have been rendered and holds the root path.
/// Additionally holds references to the [GameMap] and [Grapher] objects, should they exist of course.
/// It is meant to be used in the [Renderable] trait to render objects and generally act as a helper for rendering objects.
pub struct Renderer<'a> {
    /// The [minijinja] environment object that is used to render the templates.
    env: &'a Environment<'a>,
    /// A hashmap that tracks which objects have been rendered.
    rendered: HashMap<&'static str, HashSet<GameId>>,
    /// The path where the objects will be rendered to.
    path: String,
    /// The game map object, if it exists.
    /// It may be utilized during the rendering process to render the map.
    game_map: Option<&'a GameMap>,
    /// The grapher object, if it exists.
    /// It may be utilized during the rendering process to render a variety of graphs.
    grapher: Option<&'a Grapher>,
    /// The game state object.
    /// It is used to access the game state during rendering, especially for gathering of data for rendering of optional graphs.
    state: &'a GameState,
}

impl<'a> Renderer<'a> {
    /// Create a new Renderer with the given [Environment] and path.
    pub fn new(
        env: &'a Environment<'a>,
        path: String,
        state: &'a GameState,
        game_map: Option<&'a GameMap>,
        grapher: Option<&'a Grapher>,
    ) -> Self {
        Renderer {
            env,
            rendered: HashMap::new(),
            path,
            game_map,
            grapher,
            state,
        }
    }

    /// Returns true if the object has already been rendered.
    fn is_rendered<T: Renderable>(&self, id: GameId) -> bool {
        let rendered = self.rendered.get(T::get_subdir());
        if rendered.is_none() {
            return false;
        }
        return rendered.unwrap().contains(&id);
    }

    /// Renders the object and returns true if it was actually rendered.
    pub fn render<T: Renderable + Cullable>(&mut self, obj: &T) -> bool {
        //if it is rendered then return
        if self.is_rendered::<T>(obj.get_id()) || !obj.is_ok() {
            return false;
        }
        let path = obj.get_path(&self.path);
        let ctx = Value::from_serialize(obj);
        let template = self.env.get_template(T::get_template()).unwrap();
        let contents = template.render(ctx).unwrap();
        thread::spawn(move || {
            //IO heavy, so spawn a thread
            fs::write(path, contents).unwrap();
        });
        let rendered = self
            .rendered
            .entry(T::get_subdir())
            .or_insert(HashSet::new());
        rendered.insert(obj.get_id());
        return true;
    }

    /// Returns the root path of the rendered output
    pub fn get_path(&self) -> &str {
        &self.path
    }

    /// Returns the [Grapher] object if it exists.
    pub fn get_grapher(&self) -> Option<&Grapher> {
        self.grapher
    }

    /// Returns the [GameMap] object if it exists.
    pub fn get_map(&self) -> Option<&GameMap> {
        self.game_map
    }

    /// Returns the [GameState] object.
    pub fn get_state(&self) -> &GameState {
        self.state
    }
}

/// Trait for objects that can be rendered into a html page.
/// Since this uses [minijinja] the [serde::Serialize] trait is also needed.
/// Each object that implements this trait should have a corresponding template file in the templates folder.
pub trait Renderable: Serialize + GameObjectDerived {
    /// Returns the template file name.
    fn get_template() -> &'static str;

    /// Returns the subdirectory name where the object should be written to.
    fn get_subdir() -> &'static str;

    /// Returns the path where the object should be written to.
    fn get_path(&self, path: &str) -> String {
        format!("{}/{}/{}.html", path, Self::get_subdir(), self.get_id())
    }

    /// Renders the object and all the references of the object if they are not already rendered.
    /// This is used to render the object and add the references to the stack for rendering.
    /// The implementation should call [Renderer::render] to render the object, render whatever else it needs and add the references to the stack.
    /// It is the responsibility of the implementation to ensure that all the references are rendered.
    fn render_all(&self, stack: &mut Vec<RenderableType>, renderer: &mut Renderer);
}

/// Trait for objects that can be culled.
/// This is used to limit object serialization to a certain depth.
/// Not all [Renderable] objects need to implement this trait.
pub trait Cullable: GameObjectDerived {
    /// Set the depth of the object and performs localization.
    /// Ideally this should be called on the root object once and the depth should be propagated to all children.
    /// Also ideally should do nothing if the depth is less than or equal to the current depth.
    /// Localization of the objects should also be done here, hence the [Localizer] object.
    /// It is the responsibility of the implementation to ensure that the depth is propagated to all children.
    fn set_depth(&mut self, depth: usize, localization: &Localizer);

    /// Get the depth of the object.
    fn get_depth(&self) -> usize;

    /// Returns true if the object is ok to be rendered / serialized.
    fn is_ok(&self) -> bool {
        self.get_depth() > 0
    }
}
