use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use minijinja::{Environment, context};

use serde::Serialize;
use serde::ser::SerializeStruct;

use crate::game_object::{GameObject, SaveFileValue};
use crate::game_state::GameState;

use super::renderer::Renderable;
use super::{Character, GameObjectDerived, Shared};

pub struct Title {
    pub name: Shared<String>,
    pub deJure: Option<Shared<Title>>,
    pub deFacto: Option<Shared<Title>>,
    pub vassals: Vec<Shared<Title>>,
    pub history: HashMap<String, (Shared<Character>, Shared<String>)>
}

impl GameObjectDerived for Title{

    fn from_game_object(base: Ref<'_, GameObject>, game_state: &GameState) -> Self {
        let de_jure_id = base.get("de_jure_liege");
        let de_jure = match de_jure_id{
            Some(dejure) => Some(game_state.get_title(dejure.as_string_ref().unwrap().as_str()).unwrap().clone()),
            None => None
        };
        let de_facto_id = base.get("de_facto_liege");
        let de_facto = match de_facto_id{
            Some(defacto) => Some(game_state.get_title(defacto.as_string_ref().unwrap().as_str()).unwrap().clone()),
            None => None
        };
        let mut vassals = Vec::new();
        let vas = base.get("vassals");
        if !vas.is_none(){
            for v in base.get_object_ref("vassals").get_array(){
                vassals.push(game_state.get_title(v.as_string_ref().unwrap().as_str()).unwrap().clone());
            }
        }
        let mut history = HashMap::new();
        let hist = base.get("history").unwrap().as_object_ref().unwrap();
        for h in hist.get_keys(){
            let val = hist.get(&h);
            let character;
            let action:Shared<String>;
            match val{
                Some(&SaveFileValue::Object(ref o)) => {
                    let r = o.as_ref().borrow();
                    action = r.get("type").unwrap().as_string();
                    character = game_state.get_character(r.get_string_ref("holder").as_str()).unwrap().clone();
                },
                Some(&SaveFileValue::String(ref o)) => {
                    action = Rc::new(RefCell::new("Inherited".to_owned()));
                    character = game_state.get_character(o.as_ref().borrow().as_str()).unwrap().clone();
                }
                _ => {
                    panic!("Invalid history entry")
                }
            }
            let ent = (character, action);
            history.insert(h, ent);
        }
        Title{
            name: base.get("name").unwrap().as_string(),
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
