use std::rc::Rc;

use minijinja::context;

use serde::Serialize;
use serde::ser::SerializeStruct;

use crate::game_object::{GameObject, SaveFileValue};
use crate::game_state::GameState;

use super::renderer::Renderable;
use super::{serialize_array, Character, Cullable, DerivedRef, GameObjectDerived, Renderer, Shared};

/// A struct representing a title in the game
pub struct Title {
    id: u32,
    name: Rc<String>,
    de_jure: Option<Shared<Title>>,
    de_facto: Option<Shared<Title>>,
    vassals: Vec<Shared<Title>>,
    history: Vec<(Rc<String>, Option<Shared<Character>>, Rc<String>)>,
    depth: usize
}

///Gets the history of the title and returns a hashmap with the history entries
fn get_history(base:&GameObject, game_state:&mut GameState) -> Vec<(Rc<String>, Option<Shared<Character>>, Rc<String>)>{
    let mut history: Vec<(Rc<String>, Option<Shared<Character>>, Rc<String>)> = Vec::new();
    let hist = base.get("history");
    if hist.is_some() {
        let hist_obj = hist.unwrap().as_object().unwrap();
        for h in hist_obj.get_keys(){
            let val = hist_obj.get(&h);
            let character;
            let action:Rc<String>;
            match val{
                Some(&SaveFileValue::Object(ref o)) => {
                    let r = o;
                    action = r.get("type").unwrap().as_string();
                    let holder = r.get("holder");
                    match holder{
                        Some(h) => {
                            character = Some(game_state.get_character(h.as_string().as_str()).clone());
                        },
                        None => {
                            character = None;
                        }
                    }
                },
                Some(&SaveFileValue::String(ref o)) => {
                    action = Rc::new("Inherited".to_owned());
                    character = Some(game_state.get_character(o.as_str()).clone());
                }
                _ => {
                    panic!("Invalid history entry")
                }
            }
            let ent = (character, action);
            history.push((Rc::new(h.to_string()), ent.0, ent.1));
        }
    }
    history
}

impl GameObjectDerived for Title{

    fn from_game_object(base: &GameObject, game_state: &mut GameState) -> Self {
        //first we get the optional de_jure_liege and de_facto_liege
        let de_jure_id = base.get("de_jure_liege");
        let de_jure = match de_jure_id{
            Some(de_jure) => Some(game_state.get_title(de_jure.as_string().as_str()).clone()),
            None => None
        };
        let de_facto_id = base.get("de_facto_liege");
        let de_facto = match de_facto_id{
            Some(de_facto) => Some(game_state.get_title(de_facto.as_string().as_str()).clone()),
            None => None
        };
        let mut vassals = Vec::new();
        //if the title has vassals, we get them
        let vas = base.get("vassals");
        if !vas.is_none(){
            for v in base.get_object_ref("vassals").get_array_iter(){
                vassals.push(game_state.get_title(v.as_string().as_str()).clone());
            }
        }
        let name = base.get("name").unwrap().as_string().clone();
        let id = base.get_name().parse::<u32>().unwrap();
        let history = get_history(base, game_state);
        Title{
            name: name,
            de_jure: de_jure,
            de_facto: de_facto,
            vassals: vassals,
            history: history,
            id: id,
            depth: 0
        }
    }

    fn get_id(&self) -> u32 {
        self.id
    }

    fn dummy(id:u32) -> Self {
        Title{
            name: Rc::new("Dummy".to_owned()),
            de_jure: None,
            de_facto: None,
            vassals: Vec::new(),
            history: Vec::new(),
            id: id,
            depth: 0
        }
    }

    fn init(&mut self, base:&GameObject, game_state:&mut GameState) {
        let de_jure_id = base.get("de_jure_liege");
        self.de_jure = match de_jure_id{
            Some(de_jure) => Some(game_state.get_title(de_jure.as_string().as_str()).clone()),
            None => None
        };
        let de_facto_id = base.get("de_facto_liege");
        self.de_facto = match de_facto_id{
            Some(de_facto) => Some(game_state.get_title(de_facto.as_string().as_str()).clone()),
            None => None
        };
        let mut vassals = Vec::new();
        let vas = base.get("vassals");
        if !vas.is_none(){
            for v in base.get_object_ref("vassals").get_array_iter(){
                vassals.push(game_state.get_title(v.as_string().as_str()).clone());
            }
        }
        self.vassals = vassals;
        self.name = base.get("name").unwrap().as_string().clone();
        let history = get_history(base, game_state);
        self.history = history;
    }

    fn get_name(&self) -> Rc<String> {
        self.name.clone()
    
    }
}

impl Serialize for Title {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Title", 5)?;
        state.serialize_field("name", &self.name)?;
        if self.de_jure.is_some(){
            let de_jure = DerivedRef::from_derived(self.de_jure.as_ref().unwrap().clone());
            state.serialize_field("de_jure", &de_jure)?;
        }
        if self.de_facto.is_some(){
            let de_facto = DerivedRef::from_derived(self.de_facto.as_ref().unwrap().clone());
            state.serialize_field("de_facto", &de_facto)?;
        }
        let vassals = serialize_array(&self.vassals);
        state.serialize_field("vassals", &vassals)?;
        let mut history = Vec::new();
        for h in self.history.iter(){
            let mut o = (h.0.clone(), None, h.2.clone());
            if h.1.is_some(){
                let c = DerivedRef::from_derived(h.1.as_ref().unwrap().clone());
                o.1 = Some(c);
            }
            history.push(o);
        }
        state.serialize_field("history", &self.history)?;
        state.end()
    }
}

impl Renderable for Title {

    fn get_context(&self) -> minijinja::Value {
        return context! {title=>self};
    }

    fn get_template() -> &'static str {
        "titleTemplate.html"
    }

    fn get_subdir() -> &'static str {
        "titles"
    }

    fn render_all(&self, renderer: &mut Renderer) {
        if !renderer.render(self) {
            return;
        }
        if self.de_jure.is_some(){
            self.de_jure.as_ref().unwrap().borrow().render_all(renderer);
        }
        if self.de_facto.is_some(){
            self.de_facto.as_ref().unwrap().borrow().render_all(renderer);
        }
        for v in &self.vassals{
            v.as_ref().borrow().render_all(renderer);
        }
        for o in &self.history{
            if o.1.is_some(){
                o.1.as_ref().unwrap().borrow().render_all(renderer);
            }
        }
    }
}

impl Cullable for Title {
    fn set_depth(&mut self, depth:usize) {
        if depth <= self.depth || depth == 0{
            return;
        }
        self.depth = depth;
        if self.de_jure.is_some(){
            self.de_jure.as_ref().unwrap().borrow_mut().set_depth(depth-1);
        }
        if self.de_facto.is_some(){
            self.de_facto.as_ref().unwrap().borrow_mut().set_depth(depth-1);
        }
        for v in &self.vassals{
            v.borrow_mut().set_depth(depth-1);
        }
        for o in &self.history{
            if o.1.is_some(){
                let c = o.1.as_ref().unwrap().try_borrow_mut();
                if c.is_ok(){
                    c.unwrap().set_depth(depth-1);
                }
            }
        }
    }

    fn get_depth(&self) -> usize {
        self.depth
    }
}
