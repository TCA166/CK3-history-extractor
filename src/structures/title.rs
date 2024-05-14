use minijinja::context;

use serde::Serialize;
use serde::ser::SerializeStruct;

use crate::game_object::{GameObject, GameString, SaveFileValue};
use crate::game_state::GameState;
use crate::types::{Wrapper, WrapperMut};

use super::renderer::Renderable;
use super::{serialize_array, Character, Cullable, DerivedRef, GameId, GameObjectDerived, Renderer, Shared};

/// A struct representing a title in the game
pub struct Title {
    id: GameId,
    name: GameString,
    de_jure: Option<Shared<Title>>,
    de_facto: Option<Shared<Title>>,
    vassals: Vec<Shared<Title>>,
    history: Vec<(GameString, Option<Shared<Character>>, GameString)>,
    depth: usize
}

/// Compares two date strings in the format "YYYY.MM.DD" and returns the ordering
fn date_string_cmp(a:&str, b:&str) -> std::cmp::Ordering{
    let a_split: Vec<&str> = a.split('.').collect();
    let b_split: Vec<&str> = b.split('.').collect();
    let a_year = a_split[0].parse::<i32>().unwrap();
    let b_year = b_split[0].parse::<i32>().unwrap();
    if a_year < b_year{
        return std::cmp::Ordering::Less;
    }
    else if a_year > b_year{
        return std::cmp::Ordering::Greater;
    }
    let a_month = a_split[1].parse::<i32>().unwrap();
    let b_month = b_split[1].parse::<i32>().unwrap();
    if a_month < b_month{
        return std::cmp::Ordering::Less;
    }
    else if a_month > b_month{
        return std::cmp::Ordering::Greater;
    }
    let a_day = a_split[2].parse::<i32>().unwrap();
    let b_day = b_split[2].parse::<i32>().unwrap();
    if a_day < b_day{
        return std::cmp::Ordering::Less;
    }
    else if a_day > b_day{
        return std::cmp::Ordering::Greater;
    }
    std::cmp::Ordering::Equal
}

///Gets the history of the title and returns a hashmap with the history entries
fn get_history(base:&GameObject, game_state:&mut GameState) -> Vec<(GameString, Option<Shared<Character>>, GameString)>{
    let mut history: Vec<(GameString, Option<Shared<Character>>, GameString)> = Vec::new();
    let hist = base.get("history");
    if hist.is_some() {
        let hist_obj = hist.unwrap().as_object().unwrap();
        for h in hist_obj.get_keys(){
            let val = hist_obj.get(&h);
            let character;
            let action:GameString;
            match val{
                Some(&SaveFileValue::Object(ref o)) => {
                    if o.is_array(){
                        for entry in o.get_array_iter(){
                            let loc_action;
                            let loc_character;
                            match entry {
                                SaveFileValue::Object(ref o) => {
                                    loc_action = o.get("type").unwrap().as_string();
                                    let holder = o.get("holder");
                                    match holder{
                                        Some(h) => {
                                            loc_character = Some(game_state.get_character(&h.as_id()).clone());
                                        },
                                        None => {
                                            loc_character = None;
                                        }
                                    }
                                    
                                }
                                SaveFileValue::String(ref o) => {
                                    loc_action = GameString::wrap("Inherited".to_owned());
                                    loc_character = Some(game_state.get_character(&o.parse::<GameId>().unwrap()).clone());
                                }
                            }
                            history.push((GameString::wrap(h.to_string()), loc_character, loc_action))
                        }
                        continue; //if it's an array we handled all the adding already in the loop above
                    }
                    else{
                        action = o.get("type").unwrap().as_string();
                        let holder = o.get("holder");
                        match holder{
                            Some(h) => {
                                character = Some(game_state.get_character(&h.as_id()).clone());
                            },
                            None => {
                                character = None;
                            }
                        }
                    }
                },
                Some(&SaveFileValue::String(ref o)) => {
                    action = GameString::wrap("Inherited".to_owned());
                    character = Some(game_state.get_character(&o.parse::<GameId>().unwrap()).clone());
                }
                _ => {
                    panic!("Invalid history entry")
                }
            }
            history.push((GameString::wrap(h.to_string()), character, action));
        }
    }
    //sort history by the first element of the tuple (the date) in descending order
    history.sort_by(|a, b| date_string_cmp(a.0.as_str(), b.0.as_str()));
    history
}

impl GameObjectDerived for Title{

    fn from_game_object(base: &GameObject, game_state: &mut GameState) -> Self {
        //first we get the optional de_jure_liege and de_facto_liege
        let de_jure_id = base.get("de_jure_liege");
        let de_jure = match de_jure_id{
            Some(de_jure) => Some(game_state.get_title(&de_jure.as_id()).clone()),
            None => None
        };
        let de_facto_id = base.get("de_facto_liege");
        let de_facto = match de_facto_id{
            Some(de_facto) => Some(game_state.get_title(&de_facto.as_id()).clone()),
            None => None
        };
        let mut vassals = Vec::new();
        //if the title has vassals, we get them
        let vas = base.get("vassals");
        if !vas.is_none(){
            for v in base.get_object_ref("vassals").get_array_iter(){
                vassals.push(game_state.get_title(&v.as_id()).clone());
            }
        }
        let name = base.get("name").unwrap().as_string().clone();
        let id = base.get_name().parse::<GameId>().unwrap();
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

    fn get_id(&self) -> GameId {
        self.id
    }

    fn dummy(id:GameId) -> Self {
        Title{
            name: GameString::wrap("Dummy".to_owned()),
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
            Some(de_jure) => Some(game_state.get_title(&de_jure.as_id()).clone()),
            None => None
        };
        let de_facto_id = base.get("de_facto_liege");
        self.de_facto = match de_facto_id{
            Some(de_facto) => Some(game_state.get_title(&de_facto.as_id()).clone()),
            None => None
        };
        let mut vassals = Vec::new();
        let vas = base.get("vassals");
        if !vas.is_none(){
            for v in base.get_object_ref("vassals").get_array_iter(){
                vassals.push(game_state.get_title(&v.as_id()).clone());
            }
        }
        self.vassals = vassals;
        self.name = base.get("name").unwrap().as_string().clone();
        let history = get_history(base, game_state);
        self.history = history;
    }

    fn get_name(&self) -> GameString {
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
            self.de_jure.as_ref().unwrap().get_internal().render_all(renderer);
        }
        if self.de_facto.is_some(){
            self.de_facto.as_ref().unwrap().get_internal().render_all(renderer);
        }
        for v in &self.vassals{
            v.get_internal().render_all(renderer);
        }
        for o in &self.history{
            if o.1.is_some(){
                o.1.as_ref().unwrap().get_internal().render_all(renderer);
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
            self.de_jure.as_ref().unwrap().get_internal_mut().set_depth(depth-1);
        }
        if self.de_facto.is_some(){
            self.de_facto.as_ref().unwrap().get_internal_mut().set_depth(depth-1);
        }
        for v in &self.vassals{
            v.get_internal_mut().set_depth(depth-1);
        }
        for o in &self.history{
            if o.1.is_some(){
                let c = o.1.as_ref().unwrap().try_get_internal_mut();
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

impl Title{
    pub fn get_holder(&self) -> Option<Shared<Character>>{
        let entry = self.history.last();
        if entry.is_none(){
            return None;
        }
        entry.unwrap().1.clone()
    } 
}
