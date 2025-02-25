use std::{
    cell::Ref,
    collections::VecDeque,
    fs,
    ops::Deref,
    path::{Path, PathBuf},
    thread,
};

use derive_more::From;
use minijinja::{Environment, Value};

use serde::Serialize;

use super::{
    super::{
        game_data::GameData,
        parser::{GameId, GameState},
        structures::{
            Character, Culture, Dynasty, EntityRef, Faith, FromGameObject, GameObjectDerived,
            GameObjectEntity, GameRef, Title,
        },
        types::{HashMap, Wrapper},
    },
    graph::Grapher,
};

/// A convenience function to create a directory if it doesn't exist, and do nothing if it does.
/// Also prints an error message if the directory creation fails.
fn create_dir_maybe<P: AsRef<Path>>(name: P) {
    if let Err(err) = fs::create_dir_all(name) {
        if err.kind() != std::io::ErrorKind::AlreadyExists {
            println!("Failed to create folder: {}", err);
        }
    }
}

#[derive(From)]
enum RenderableType {
    Character(GameRef<Character>),
    Dynasty(GameRef<Dynasty>),
    Title(GameRef<Title>),
    Faith(GameRef<Faith>),
    Culture(GameRef<Culture>),
}

impl TryFrom<&EntityRef> for RenderableType {
    type Error = ();

    fn try_from(value: &EntityRef) -> Result<Self, Self::Error> {
        match value {
            EntityRef::Character(c) => Ok(c.clone().into()),
            EntityRef::Dynasty(d) => Ok(d.clone().into()),
            EntityRef::Title(t) => Ok(t.clone().into()),
            EntityRef::Faith(f) => Ok(f.clone().into()),
            EntityRef::Culture(c) => Ok(c.clone().into()),
            _ => Err(()),
        }
    }
}

impl RenderableType {
    fn get_id(&self) -> GameId {
        match self {
            RenderableType::Character(c) => c.get_internal().get_id(),
            RenderableType::Dynasty(d) => d.get_internal().get_id(),
            RenderableType::Title(t) => t.get_internal().get_id(),
            RenderableType::Faith(f) => f.get_internal().get_id(),
            RenderableType::Culture(c) => c.get_internal().get_id(),
        }
    }

    fn get_subdir(&self) -> &'static str {
        match self {
            RenderableType::Character(_) => Character::get_subdir(),
            RenderableType::Dynasty(_) => Dynasty::get_subdir(),
            RenderableType::Title(_) => Title::get_subdir(),
            RenderableType::Faith(_) => Faith::get_subdir(),
            RenderableType::Culture(_) => Culture::get_subdir(),
        }
    }

    fn is_initialized(&self) -> bool {
        match self {
            RenderableType::Character(c) => c.get_internal().inner().is_some(),
            RenderableType::Dynasty(d) => d.get_internal().inner().is_some(),
            RenderableType::Title(t) => t.get_internal().inner().is_some(),
            RenderableType::Faith(f) => f.get_internal().inner().is_some(),
            RenderableType::Culture(c) => c.get_internal().inner().is_some(),
        }
    }
}

/// A struct that renders objects into html pages.
/// It is meant to be used as a worker object that collects objects and renders them all at once.
/// The objects are rendered in a BFS order, with the depth of the objects being determined by the BFS algorithm.
pub struct Renderer<'a> {
    depth_map: HashMap<EntityRef, usize>,
    /// The path where the objects will be rendered to.
    /// This usually takes the form of './{username}'s history/'.
    path: &'a Path,
    /// The loaded game data object.
    data: &'a GameData,
    /// The grapher object, if it exists.
    /// It may be utilized during the rendering process to render a variety of graphs.
    grapher: Option<&'a Grapher>,
    /// The game state object.
    /// It is used to access the game state during rendering, especially for gathering of data for rendering of optional graphs.
    state: &'a GameState,
    initial_depth: usize,
}

impl<'a> Renderer<'a> {
    /// Create a new Renderer.
    /// [create_dir_maybe] is called on the path to ensure that the directory exists, and the subdirectories are created.
    ///
    /// # Arguments
    ///
    /// * `path` - The root path where the objects will be rendered to. Usually takes the form of './{username}'s history/'.
    /// * `state` - The game state object.
    /// * `game_map` - The game map object, if it exists.
    /// * `grapher` - The grapher object, if it exists.
    /// * `initial_depth` - The initial depth of the objects that are added to the renderer.
    ///
    /// # Returns
    ///
    /// A new Renderer object.
    pub fn new(
        path: &'a Path,
        state: &'a GameState,
        data: &'a GameData,
        grapher: Option<&'a Grapher>,
        initial_depth: usize,
    ) -> Self {
        create_dir_maybe(path);
        create_dir_maybe(path.join(Character::get_subdir()));
        create_dir_maybe(path.join(Dynasty::get_subdir()));
        create_dir_maybe(path.join(Title::get_subdir()));
        create_dir_maybe(path.join(Faith::get_subdir()));
        create_dir_maybe(path.join(Culture::get_subdir()));
        Renderer {
            depth_map: HashMap::default(),
            path,
            data,
            grapher,
            state,
            initial_depth,
        }
    }

    /// Renders the [Renderable] object.
    fn render<T: Renderable>(&self, obj: Ref<T>, env: &Environment<'_>) {
        //render the object
        let template = env.get_template(T::get_template()).unwrap();
        let path = obj.get_path(self.path);
        obj.render(&self.path, &self.state, self.grapher, self.data);
        let contents = template.render(obj.deref()).unwrap();
        thread::spawn(move || {
            //IO heavy, so spawn a thread
            fs::write(path, contents).unwrap();
        });
    }

    /// Renders the [RenderableType] object.
    fn render_enum(&self, obj: &RenderableType, env: &Environment<'_>) {
        if !obj.is_initialized() {
            return;
        }
        match obj {
            RenderableType::Character(obj) => self.render(obj.get_internal(), env),
            RenderableType::Dynasty(obj) => self.render(obj.get_internal(), env),
            RenderableType::Title(obj) => self.render(obj.get_internal(), env),
            RenderableType::Faith(obj) => self.render(obj.get_internal(), env),
            RenderableType::Culture(obj) => self.render(obj.get_internal(), env),
        }
    }

    /// Adds an object to the renderer, and returns the number of objects that were added.
    /// This method uses a BFS algorithm to determine the depth of the object.
    pub fn add_object<G: GameObjectDerived>(&mut self, obj: &G) -> usize {
        // BFS with depth https://stackoverflow.com/a/31248992/12520385
        let mut queue: VecDeque<Option<EntityRef>> = VecDeque::new();
        // FIXME this makes obj not rendered
        obj.get_references(&mut queue);
        let mut res = queue.len();
        queue.push_back(None);
        // algorithm determined depth
        let mut alg_depth = self.initial_depth;
        while let Some(obj) = queue.pop_front() {
            res += 1;
            if let Some(obj) = obj {
                if let Some(stored_depth) = self.depth_map.get_mut(&obj) {
                    if alg_depth > *stored_depth {
                        *stored_depth = alg_depth;
                        obj.get_references(&mut queue);
                    }
                } else {
                    obj.get_references(&mut queue);
                    self.depth_map.insert(obj, alg_depth);
                }
            } else {
                alg_depth -= 1;
                if alg_depth == 0 {
                    break;
                }
                queue.push_back(None);
                if queue.front().unwrap().is_none() {
                    break;
                }
            }
        }
        return res;
    }

    /// Renders all the objects that have been added to the renderer.
    /// This method consumes the renderer object.
    ///
    /// # Arguments
    ///
    /// * `env` - The [Environment] object that is used to render the templates.
    ///
    /// # Returns
    ///
    /// The number of objects that were rendered.
    pub fn render_all(self, env: &mut Environment<'_>) -> usize {
        let mut global_depth_map = HashMap::default();
        for (obj, value) in self.depth_map.iter() {
            if let Ok(obj) = RenderableType::try_from(obj) {
                global_depth_map
                    .entry(obj.get_subdir())
                    .or_insert(HashMap::default())
                    .insert(obj.get_id(), *value);
            }
        }
        env.add_global("depth_map", Value::from_serialize(global_depth_map));
        for obj in self.depth_map.keys() {
            if let Ok(obj) = obj.try_into() {
                self.render_enum(&obj, env);
            }
        }
        env.remove_global("depth_map");
        return self.depth_map.len();
    }
}

pub trait ProceduralPath {
    fn get_subdir() -> &'static str;
}

pub trait GetPath {
    fn get_path(&self, path: &Path) -> PathBuf;
}

impl<T: GameObjectDerived + ProceduralPath + FromGameObject> GetPath for GameObjectEntity<T> {
    fn get_path(&self, path: &Path) -> PathBuf {
        let mut buf = path.join(T::get_subdir());
        buf.push(format!("{}.html", self.get_id()));
        buf
    }
}

/// Trait for objects that can be rendered into a html page.
/// Since this uses [minijinja] the [serde::Serialize] trait is also needed.
/// Each object that implements this trait should have a corresponding template file in the templates folder.
pub trait Renderable: Serialize + GetPath {
    /// Returns the template file name.
    /// This method is used to retrieve the template from the [Environment] object in the [Renderer] object.
    fn get_template() -> &'static str;

    /// Renders all the objects that are related to this object.
    /// For example: graphs, maps, etc.
    /// This is where your custom rendering logic should go.
    ///
    /// # Arguments
    ///
    /// * `path` - The root output path of the renderer.
    /// * `game_state` - The game state object.
    /// * `grapher` - The grapher object, if it exists.
    /// * `data` - The game data object.
    ///
    /// # Default Implementation
    ///
    /// The default implementation does nothing. It is up to the implementing object to override this method.
    #[allow(unused_variables)]
    fn render(
        &self,
        path: &Path,
        game_state: &GameState,
        grapher: Option<&Grapher>,
        data: &GameData,
    ) {
    }
}
