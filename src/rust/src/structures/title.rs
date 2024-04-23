use std::collections::HashMap;
use std::rc::Rc;

use minijinja::{Environment, context};

use serde::Serialize;
use serde::ser::SerializeStruct;

use crate::game_object::{GameObject, SaveFileValue};
use crate::game_state::GameState;

use super::renderer::Renderable;
use super::{Character, GameObjectDerived};

pub struct Title {
    pub name: Rc<String>,
    pub deJure: Option<Rc<Title>>,
    pub deFacto: Option<Rc<Title>>,
    pub vassals: Vec<Rc<Title>>,
    pub history: HashMap<String, (Rc<Character>, Rc<String>)>
}

impl GameObjectDerived for Title{

    fn from_game_object(base: &GameObject, game_state: &GameState) -> Self {
        let de_jure_id = base.get("de_jure_liege");
        let de_jure = match de_jure_id{
            Some(dejure) => Some(game_state.get_title(dejure.as_string().unwrap().as_str()).unwrap().clone()),
            None => None
        };
        let de_facto_id = base.get("de_facto_liege");
        let de_facto = match de_facto_id{
            Some(defacto) => Some(game_state.get_title(defacto.as_string().unwrap().as_str()).unwrap().clone()),
            None => None
        };
        let mut vassals = Vec::new();
        for v in base.get("vassals").unwrap().as_array().unwrap(){
            vassals.push(game_state.get_title(v.as_string().unwrap().as_str()).unwrap().clone());
        }
        let mut history = HashMap::new();
        let hist = base.get("history").unwrap().as_object().unwrap();
        for h in hist.get_keys(){
            let val = hist.get(&h);
            let character;
            let action;
            match val{
                Some(&SaveFileValue::Object(ref o)) => {
                    action = o.get("type").unwrap().as_string().unwrap();
                    character = game_state.get_character(o.get("holder").unwrap().as_string().unwrap().as_str()).unwrap().clone();
                },
                Some(&SaveFileValue::String(ref o)) => {
                    action = Rc::new("Inherited".to_owned());
                    character = game_state.get_character(o.as_str()).unwrap().clone();
                }
                _ => {
                    panic!("Invalid history entry")
                }
            }
            let ent = (character, action);
            history.insert(h, ent);
        }
        Title{
            name: base.get("name").unwrap().as_string().unwrap(),
            deJure: de_jure,
            deFacto: de_facto,
            vassals: vassals,
            history: history
        }
    }

    fn type_name() -> &'static str {
        "title"
    }

}

impl Serialize for Title {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Title", 5)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("deJure", &self.deJure)?;
        state.serialize_field("deFacto", &self.deFacto)?;
        state.serialize_field("vassals", &self.vassals)?;
        state.serialize_field("history", &self.history)?;
        state.end()
    }
}

impl Renderable for Title {
    fn render(&self, env: &Environment, template_name: &'static String) -> String {
        let ctx = context! {title=>self};
        env.get_template(template_name).unwrap().render(&ctx).unwrap()   
    }
}
