use std::{fs, thread};

use minijinja::{Environment, Value};

use serde::Serialize;

use super::{
    super::{
        parser::{GameId, GameState},
        structures::{Character, Culture, Dynasty, Faith, GameObjectDerived, Title},
        types::{HashMap, HashSet, RefOrRaw, Wrapper},
    },
    graph::Grapher,
    map::GameMap,
    RenderableType,
};

/// A convenience function to create a directory if it doesn't exist, and do nothing if it does.
/// Also prints an error message if the directory creation fails.
fn create_dir_maybe(name: &str) {
    if let Err(err) = fs::create_dir_all(name) {
        if err.kind() != std::io::ErrorKind::AlreadyExists {
            println!("Failed to create folder: {}", err);
        }
    }
}

/// A struct that renders objects into html pages.
/// It holds a reference to the [Environment] that is used to render the templates, tracks which objects have been rendered and holds the root path.
/// Additionally holds references to the [GameMap] and [Grapher] objects, should they exist of course.
/// It is meant to be used as a worker object that renders objects into html pages.
pub struct Renderer<'a> {
    /// The [minijinja] environment object that is used to render the templates.
    env: &'a Environment<'a>,
    /// A hashmap that tracks which objects have been rendered.
    rendered: HashMap<&'static str, HashSet<GameId>>,
    /// The path where the objects will be rendered to.
    /// This usually takes the form of './{username}'s history/'.
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
    /// [create_dir_maybe] is called on the path to ensure that the directory exists, and the subdirectories are created.
    /// 
    /// # Arguments
    /// 
    /// * `env` - The [Environment] object that is used to render the templates.
    /// * `path` - The root path where the objects will be rendered to. Usually takes the form of './{username}'s history/'.
    /// * `state` - The game state object.
    /// * `game_map` - The game map object, if it exists.
    /// * `grapher` - The grapher object, if it exists.
    /// 
    /// # Returns
    /// 
    /// A new Renderer object.
    pub fn new(
        env: &'a Environment<'a>,
        path: String,
        state: &'a GameState,
        game_map: Option<&'a GameMap>,
        grapher: Option<&'a Grapher>,
    ) -> Self {
        create_dir_maybe(&path);
        create_dir_maybe(format!("{path}/{}", Character::get_subdir()).as_str());
        create_dir_maybe(format!("{path}/{}", Dynasty::get_subdir()).as_str());
        create_dir_maybe(format!("{path}/{}", Title::get_subdir()).as_str());
        create_dir_maybe(format!("{path}/{}", Faith::get_subdir()).as_str());
        create_dir_maybe(format!("{path}/{}", Culture::get_subdir()).as_str());
        Renderer {
            env,
            rendered: HashMap::default(),
            path,
            game_map,
            grapher,
            state,
        }
    }

    /// Returns true if the object has already been rendered.
    fn is_rendered<T: Renderable>(&self, id: GameId) -> bool {
        if let Some(rendered) = self.rendered.get(T::get_subdir()) {
            return rendered.contains(&id);
        }
        return false;
    }

    /// Renders the object and returns true if it was actually rendered.
    /// If the object is already rendered or is not ok to be rendered, then it returns false.
    /// If the object was rendered it calls the [Renderable::render] method on the object and [Renderable::append_ref] on the object.
    fn render<T: Renderable + Cullable>(
        &mut self,
        obj: RefOrRaw<T>,
        stack: &mut Vec<RenderableType>,
    ) -> bool {
        //if it is rendered then return
        if self.is_rendered::<T>(obj.get_id()) || !obj.is_ok() {
            return false;
        }
        let path = obj.get_path(&self.path);
        let ctx = Value::from_serialize(&obj);
        let template = self.env.get_template(T::get_template()).unwrap();
        let contents = template.render(ctx).unwrap();
        thread::spawn(move || {
            //IO heavy, so spawn a thread
            fs::write(path, contents).unwrap();
        });
        obj.render(&self.path, &self.state, self.grapher, self.game_map);
        obj.append_ref(stack);
        let rendered = self
            .rendered
            .entry(T::get_subdir())
            .or_insert(HashSet::default());
        rendered.insert(obj.get_id());
        return true;
    }

    /// Renders a renderable enum object.
    /// Calls [Renderer::render] on the object if it is not already rendered.
    fn render_enum(&mut self, obj: &RenderableType, stack: &mut Vec<RenderableType>) -> bool {
        match obj {
            RenderableType::Character(obj) => self.render(obj.get_internal(), stack),
            RenderableType::Dynasty(obj) => self.render(obj.get_internal(), stack),
            RenderableType::Title(obj) => self.render(obj.get_internal(), stack),
            RenderableType::Faith(obj) => self.render(obj.get_internal(), stack),
            RenderableType::Culture(obj) => self.render(obj.get_internal(), stack),
        }
    }

    /// Renders all the objects that are related to the given object.
    /// It uses a stack to keep track of the objects that need to be rendered.
    /// 
    /// # Method
    /// 
    /// This method renders templates for given objects and the necessary graphics.
    /// 
    /// ## Template Rendering
    /// 
    /// First a corresponding template is retrieved from the [Environment] object using
    /// the template name given by [Renderable::get_template].
    /// Then the object is serialized (using [serde::Serialize]) into a [minijinja::Value] object.
    /// Using this value object the template is rendered and the contents are written to a file 
    /// using the path given by [Renderable::get_path].
    /// 
    /// ## Object Rendering
    /// 
    /// In order to ensure all the necessary objects for the template to display correctly are rendered,
    /// the [Renderable::render] method is called on the object.
    /// This method is meant to render all the graphics that are related to the object.
    /// 
    /// ## Related Objects
    /// 
    /// The [Renderable::append_ref] method is called on the object to append all the related objects to the stack.
    /// This is done to ensure that all the related objects are rendered and the process is repeated for all the objects.
    /// 
    /// # Returns
    /// 
    /// The number of objects that were rendered.
    pub fn render_all<T: Renderable + Cullable>(&mut self, obj: &T) -> u64 {
        let mut stack: Vec<RenderableType> = Vec::new();
        if !self.render(RefOrRaw::Raw(obj), &mut stack) {
            return 0;
        }
        let mut counter = 1;
        while let Some(obj) = stack.pop() {
            if self.render_enum(&obj, &mut stack) {
                counter += 1;
            }
        }
        return counter;
    }
}

/// Trait for objects that can be rendered into a html page.
/// Since this uses [minijinja] the [serde::Serialize] trait is also needed.
/// Each object that implements this trait should have a corresponding template file in the templates folder.
pub trait Renderable: Serialize + GameObjectDerived {
    /// Returns the template file name.
    /// This method is used to retrieve the template from the [Environment] object in the [Renderer] object.
    fn get_template() -> &'static str;

    /// Returns the subdirectory name where the rendered template should be written to.
    /// This method is used to create a subdirectory in the root output path, and by the [Renderable::get_path] method.
    fn get_subdir() -> &'static str;

    /// Returns the path where the rendered template should be written to.
    /// 
    /// # Arguments
    /// 
    /// * `path` - The root output path of the renderer.
    /// 
    /// # Default Implementation
    /// 
    /// The default implementation returns a path in the format: `{path}/{subdir}/{id}.html`.
    /// Subdir is returned by [Renderable::get_subdir] and id is returned by [GameObjectDerived::get_id].
    /// This can be of course overridden by the implementing object.
    /// 
    /// # Returns
    /// 
    /// The full path where the object should be written to.
    fn get_path(&self, path: &str) -> String {
        format!("{}/{}/{}.html", path, Self::get_subdir(), self.get_id())
    }

    /// Renders all the objects that are related to this object.
    /// For example: graphs, maps, etc.
    /// This is where your custom rendering logic should go.
    /// 
    /// # Arguments
    /// 
    /// * `path` - The root output path of the renderer.
    /// * `game_state` - The game state object.
    /// * `grapher` - The grapher object, if it exists.
    /// * `map` - The game map object, if it exists.
    /// 
    /// # Default Implementation
    /// 
    /// The default implementation does nothing. It is up to the implementing object to override this method.
    #[allow(unused_variables)]
    fn render(
        &self,
        path: &str,
        game_state: &GameState,
        grapher: Option<&Grapher>,
        map: Option<&GameMap>,
    ) {
    }

    /// Appends all the related objects to the stack.
    /// This is used to ensure that all the related objects are rendered.
    /// If you have a reference to another object that should be rendered, then you should append it to the stack.
    /// Failure to do so will result in broken links in the rendered html pages.
    fn append_ref(&self, stack: &mut Vec<RenderableType>);
}

/// Trait for objects that can be culled.
/// This is used to limit object serialization to a certain depth.
/// Not all [Renderable] objects need to implement this trait.
pub trait Cullable: GameObjectDerived {
    /// Set the depth of the object and performs localization.
    /// Ideally this should be called on the root object once and the depth should be propagated to all children.
    /// Also ideally should do nothing if the depth is less than or equal to the current depth.
    /// It is the responsibility of the implementation to ensure that the depth is propagated to all children.
    fn set_depth(&mut self, depth: usize);

    /// Get the depth of the object.
    fn get_depth(&self) -> usize;

    /// Returns true if the object is ok to be rendered / serialized.
    fn is_ok(&self) -> bool {
        self.get_depth() > 0
    }
}
