use minijinja::context;
use serde::Serialize;
use serde::ser::SerializeStruct;
use crate::graph::Grapher;
use crate::renderer::{Cullable, Renderable, Renderer};
use super::{Character, DerivedRef, DummyInit, GameId, GameObjectDerived, Shared};
use crate::game_object::{GameObject, GameString};
use crate::game_state::GameState;
use crate::localizer::Localizer;
use crate::map::GameMap;
use crate::types::{Wrapper, WrapperMut};

/// A struct representing a faith in the game
pub struct Faith {
    id: GameId,
    name: Option<GameString>,
    tenets: Vec<GameString>,
    head: Option<Shared<Character>>,
    fervor: f32,
    doctrines: Vec<GameString>,
    depth: usize,
    localized: bool,
    name_localized: bool,
}

/// Gets the head of the faith
fn get_head(base:&GameObject, game_state:&mut crate::game_state::GameState) -> Option<Shared<Character>>{
    let current = base.get("religious_head");
    if current.is_some(){
        let title = game_state.get_title(&current.unwrap().as_id());
        return title.get_internal().get_holder();
    }
    None
}

/// Gets the tenets of the faith and appends them to the tenets vector
fn get_tenets(tenets:&mut Vec<GameString>, array:&GameObject){
    for t in array.get_array_iter(){
        let s = t.as_string();
        if s.contains("tenet"){
            tenets.push(s);
        }
    }
}

/// Gets the doctrines of the faith and appends them to the doctrines vector
fn get_doctrines(doctrines:&mut Vec<GameString>, array:&GameObject){
    for d in array.get_array_iter(){
        let s = d.as_string();
        if !s.contains("tenet") {
            doctrines.push(s);
        }
    }
}

/// Gets the name of the faith
fn get_name(base:&GameObject) -> GameString{
    let node = base.get("name");
    if node.is_some(){
        return node.unwrap().as_string();
    }
    else{
        base.get("template").unwrap().as_string()
    }
}

impl DummyInit for Faith {
    fn dummy(id:GameId) -> Self {
        Faith{
            name: None,
            tenets: Vec::new(),
            head: None, //trying to create a dummy character here caused a fascinating stack overflow because of infinite recursion
            fervor: 0.0,
            doctrines: Vec::new(),
            id: id,
            depth: 0,
            localized: false,
            name_localized: false
        }
    }

    fn init(&mut self, base: &GameObject, game_state: &mut GameState) {
        let doctrines_array = base.get("doctrine").unwrap().as_object().unwrap();
        get_tenets(&mut self.tenets, doctrines_array);
        self.head.clone_from(&get_head(&base, game_state));
        get_doctrines(&mut self.doctrines, doctrines_array);
        self.name = Some(get_name(&base));
        self.fervor = base.get("fervor").unwrap().as_string().parse::<f32>().unwrap();
    }
}

impl GameObjectDerived for Faith {
    fn get_id(&self) -> GameId {
        self.id
    }

    fn get_name(&self) -> GameString {
        self.name.as_ref().unwrap().clone()
    }
}

impl Serialize for Faith {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Faith", 5)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("tenets", &self.tenets)?;
        if self.head.is_some(){
            let head = DerivedRef::<Character>::from_derived(self.head.as_ref().unwrap().clone());
            state.serialize_field("head", &head)?;
        }
        state.serialize_field("fervor", &self.fervor)?;
        state.serialize_field("doctrines", &self.doctrines)?;
        state.end()
    }
}

impl Renderable for Faith {
    fn get_context(&self) -> minijinja::Value {
        context!{faith=>self}
    }
    
    fn get_template() -> &'static str {
        "faithTemplate.html"
    }

    fn get_subdir() -> &'static str {
        "faiths"
    }

    fn render_all(&self, renderer: &mut Renderer, game_map:Option<&GameMap>, grapher: Option<&Grapher>) {
        if !renderer.render(self){
            return;
        }
        if grapher.is_some(){
            let path = format!("{}/faiths/{}.svg", renderer.get_path(), self.id);
            grapher.unwrap().create_faith_graph(self.id, &path);
        }
        if self.head.is_some(){
            self.head.as_ref().unwrap().get_internal().render_all(renderer, game_map, grapher);
        }
    }
}

impl Cullable for Faith {
    fn get_depth(&self) -> usize {
        self.depth
    }

    fn set_depth(&mut self, depth: usize, localization:&Localizer) {
        if depth <= self.depth && depth != 0{
            return;
        }
        if !self.name_localized{
            self.name = Some(localization.localize(self.name.as_ref().unwrap().as_str()));
            self.name_localized = true;
        }
        if depth == 0{
            return;
        }
        if !self.localized{
            for tenet in self.tenets.iter_mut(){
                *tenet = localization.localize(tenet.as_str());
            }
            for doctrine in self.doctrines.iter_mut(){
                *doctrine = localization.localize(doctrine.as_str());
            }
            self.localized = true;
        }
        self.depth = depth;
        if self.head.is_some(){
            let o = self.head.as_ref().unwrap().try_get_internal_mut();
            if o.is_ok(){
                o.unwrap().set_depth(depth-1, localization);
            }
        }
    }
}
