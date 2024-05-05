use minijinja::{Environment, context};
use std::cell::Ref;
use std::rc::Rc;
use serde::Serialize;
use serde::ser::SerializeStruct;
use super::{Character, Cullable, GameObjectDerived, Shared};
use super::renderer::Renderable;
use crate::game_object::GameObject;

pub struct Faith {
    id: u32,
    name: Shared<String>,
    tenets: Vec<Shared<String>>,
    head: Option<Shared<Character>>,
    fervor: f32,
    doctrines: Vec<Shared<String>>,
    depth: usize
}

fn get_head(base:&Ref<'_, GameObject>, game_state:&mut crate::game_state::GameState) -> Option<Shared<Character>>{
    let current = base.get("head");
    if current.is_some(){
        return Some(game_state.get_character(current.unwrap().as_string_ref().unwrap().as_str()));
    }
    None
}

fn get_tenets(tenets:&mut Vec<Shared<String>>, base:&Ref<'_, GameObject>){
    let tenets_obj = base.get("tenets");
    if tenets_obj.is_some(){
        for t in tenets_obj.unwrap().as_object_ref().unwrap().get_array_iter(){
            tenets.push(t.as_string());
        }
    }
}

fn get_doctrines(doctrines:&mut Vec<Shared<String>>, base:&Ref<'_, GameObject>){
    let doctrines_obj = base.get("doctrines");
    if doctrines_obj.is_some(){
        for d in doctrines_obj.unwrap().as_object_ref().unwrap().get_array_iter(){
            doctrines.push(d.as_string());
        }
    }
}

fn get_name(base:&Ref<'_, GameObject>) -> Shared<String>{
    let node = base.get("name");
    if node.is_some(){
        return node.unwrap().as_string();
    }
    else{
        base.get("template").unwrap().as_string()
    }
}

impl GameObjectDerived for Faith {
    fn from_game_object(base:Ref<'_, GameObject>, game_state:&mut crate::game_state::GameState) -> Self {
        let mut tenets = Vec::new();
        get_tenets(&mut tenets, &base);
        let mut doctrines = Vec::new();
        get_doctrines(&mut doctrines, &base);
        Faith{
            name: get_name(&base),
            tenets: tenets,
            head: get_head(&base, game_state),
            fervor: base.get("fervor").unwrap().as_string_ref().unwrap().parse::<f32>().unwrap(),
            doctrines: doctrines,
            id: base.get_name().parse::<u32>().unwrap(),
            depth: 0
        }
    }

    fn dummy(id:u32) -> Self {
        Faith{
            name: Rc::new("".to_owned().into()),
            tenets: Vec::new(),
            head: None, //trying to create a dummy character here caused a fascinating stack overflow because of infinite recursion
            fervor: 0.0,
            doctrines: Vec::new(),
            id: id,
            depth: 0
        }
    }

    fn init(&mut self, base: Ref<'_, GameObject>, game_state: &mut crate::game_state::GameState) {
        get_tenets(&mut self.tenets, &base);
        self.head.clone_from(&get_head(&base, game_state));
        get_doctrines(&mut self.doctrines, &base);
        self.name = get_name(&base);
        self.fervor = base.get("fervor").unwrap().as_string_ref().unwrap().parse::<f32>().unwrap();
    }

    fn get_id(&self) -> u32 {
        self.id
    }

    fn get_name(&self) -> Shared<String> {
        self.name.clone()
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
        state.serialize_field("head", &self.head)?;
        state.serialize_field("fervor", &self.fervor)?;
        state.serialize_field("doctrines", &self.doctrines)?;
        state.end()
    }
}

impl Renderable for Faith {
    fn render(&self, env: &Environment) -> Option<String> {
        if self.depth == 0{
            return None;
        }
        let ctx = context! {faith=>self};
        Some(env.get_template("faithTemplate.html").unwrap().render(&ctx).unwrap())
    }
}

impl Cullable for Faith {
    fn get_depth(&self) -> usize {
        self.depth
    }

    fn set_depth(&mut self, depth: usize) {
        if depth <= self.depth || depth == 0{
            return;
        }
        self.depth = depth;
        if self.head.is_some(){
            self.head.as_ref().unwrap().borrow_mut().set_depth(depth-1);
        }
    }
}
