use minijinja::context;

use serde::Serialize;
use serde::ser::SerializeStruct;
use super::renderer::Renderable;
use super::{serialize_array, Character, Cullable, Culture, DerivedRef, DummyInit, Faith, GameId, GameObjectDerived, Renderer, Shared};
use crate::game_object::{GameObject, GameString, SaveFileValue};
use crate::game_state::GameState;
use crate::localizer::Localizer;
use crate::types::{Wrapper, WrapperMut};

pub struct Dynasty{
    id: GameId,
    parent: Option<Shared<Dynasty>>,
    name: Option<GameString>,
    members: GameId,
    houses: GameId,
    prestige_tot: f32,
    prestige: f32,
    perks: Vec<(GameString, u8)>,
    leaders: Vec<Shared<Character>>,
    found_date: Option<GameString>,
    depth: usize,
    localized:bool,
    name_localized: bool,
}

impl Dynasty {
    /// Gets the faith of the dynasty.
    /// Really this is just the faith of the current house leader.
    pub fn get_faith(&self) -> Option<Shared<Faith>> {
        self.leaders.last().unwrap().get_internal().get_faith()
    }

    /// Gets the culture of the dynasty.
    /// Really this is just the culture of the current house leader.
    pub fn get_culture(&self) -> Option<Shared<Culture>> {
        self.leaders.last().unwrap().get_internal().get_culture()
    }

    pub fn register_house(&mut self){
        self.houses += 1;
    }

    pub fn register_member(&mut self){
        self.members += 1;
        if self.parent.as_ref().is_some(){
            let mut p = self.parent.as_ref().unwrap().try_get_internal_mut();
            if p.is_ok(){
                p.as_mut().unwrap().register_member();
            }
        }
    }
}

///Gets the perks of the dynasty and appends them to the perks vector
fn get_perks(perks:&mut Vec<(GameString, u8)>, base:&GameObject){
    let perks_obj = base.get("perk");
    if perks_obj.is_some(){
        for p in perks_obj.unwrap().as_object().unwrap().get_array_iter(){
            let perk = p.as_string();
            //get the split perk by the second underscore
            let mut i:u8 = 0;
            let mut key: Option<&str> = None;
            let mut val: u8 = 0;
            for el in perk.rsplitn(2, '_'){
                if i == 0{
                    val = el.parse::<u8>().unwrap();
                } else {
                    key = Some(el);
                }
                i += 1;
            }
            if key.is_none(){
                continue;
            }
            let mut added = false;
            for perk in perks.iter_mut(){
                if perk.0.as_str() == key.unwrap(){
                    if perk.1 < val{
                        perk.1 = val;
                    }
                    added = true;
                    break;
                }
            }
            if added{
                continue;
            }
            //if the perk is not found, add it
            perks.push((GameString::wrap(key.unwrap().to_owned()), val));
        }
    }
}

///Gets the leaders of the dynasty and appends them to the leaders vector
fn get_leaders(leaders:&mut Vec<Shared<Character>>, base:&GameObject, game_state:&mut GameState){
    let leaders_obj = base.get("historical");
    if leaders_obj.is_some(){
        for l in leaders_obj.unwrap().as_object().unwrap().get_array_iter(){
            leaders.push(game_state.get_character(&l.as_id()).clone());
        }
    }
}

///Gets the prestige of the dynasty and returns a tuple with the total prestige and the current prestige
fn get_prestige(base:&GameObject) -> (f32, f32){
    let currency = base.get("prestige");
    let mut prestige_tot = 0.0;
    let mut prestige = 0.0;
    if currency.is_some(){
        let o = currency.unwrap().as_object().unwrap();
        match o.get("accumulated").unwrap() {
            SaveFileValue::Object(ref o) => {
                prestige_tot = o.get_string_ref("value").parse::<f32>().unwrap();
            },
            SaveFileValue::String(ref o) => {
                prestige_tot = o.parse::<f32>().unwrap();
            },
        }
        match o.get("currency") {
            Some(v) => match v {
                SaveFileValue::Object(ref o) => {
                    prestige = o.get_string_ref("value").parse::<f32>().unwrap();
                },
                SaveFileValue::String(ref o) => {
                    prestige = o.parse::<f32>().unwrap();
                },
            },
            None => {}
        }
    }
    (prestige_tot, prestige)
}

///Gets the parent dynasty of the dynasty
fn get_parent(base:&GameObject, game_state:&mut GameState) -> Option<Shared<Dynasty>>{
    let parent_id = base.get("dynasty");
    match parent_id {
        None => None,
        k => {
            let p = game_state.get_dynasty(&k.unwrap().as_id()).clone();
            let m = p.try_get_internal_mut();
            if m.is_err(){
                return None;
            }
            m.unwrap().register_house();
            Some(p)
        }
    }
}

fn get_name(base:&GameObject, parent:Option<Shared<Dynasty>>) -> Option<GameString>{
    let mut n = base.get("name");
    if n.is_none(){
        n = base.get("localized_name");
        if n.is_none(){
            if parent.is_none(){
                //println!("{:?}", base);
                return None;
            }
            let p = parent.as_ref().unwrap().get_internal();
            if p.name.is_none(){
                return None;
            }
            //this may happen for dynasties with a house with the same name
            return Some(p.name.as_ref().unwrap().clone());
        }
    }
    Some(n.unwrap().as_string())
}

fn get_date(base:&GameObject) -> Option<GameString>{
    let date = base.get("found_date");
    if date.is_none(){
        return None;
    }
    Some(date.unwrap().as_string())
}

impl DummyInit for Dynasty {
    fn dummy(id:GameId) -> Self {
        Dynasty{
            name: None,
            parent: None,
            members: 0,
            houses: 0,
            prestige_tot: 0.0,
            prestige: 0.0,
            perks: Vec::new(),
            leaders: Vec::new(),
            found_date: None,
            id: id,
            depth: 0,
            localized:false,
            name_localized: false
        }
    }

    fn init(&mut self, base:&GameObject, game_state:&mut GameState) {
        get_perks(&mut self.perks, &base);
        get_leaders(&mut self.leaders, &base, game_state);
        let res = get_prestige(&base);
        if self.prestige_tot == 0.0{
            self.prestige_tot = res.0;
            self.prestige = res.1;
        }
        self.parent = get_parent(&base, game_state);
        let name = get_name(&base, self.parent.clone());
        if self.name.is_none() {
            self.name = name;
        }
        self.found_date = get_date(&base);
    }
}

impl GameObjectDerived for Dynasty {
    fn get_id(&self) -> GameId {
        self.id
    }

    fn get_name(&self) -> GameString {
        if self.name.is_none(){
            return GameString::wrap("Unknown".to_owned());
        }
        self.name.as_ref().unwrap().clone()
    }
}

impl Serialize for Dynasty {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Dynasty", 9)?;
        if self.parent.as_ref().is_some(){
            let parent = DerivedRef::<Dynasty>::from_derived(self.parent.as_ref().unwrap().clone());
            state.serialize_field("parent", &parent)?;
        }
        state.serialize_field("name", &self.name)?;
        state.serialize_field("members", &self.members)?;
        state.serialize_field("houses", &self.houses)?;
        state.serialize_field("prestige_tot", &self.prestige_tot)?;
        state.serialize_field("prestige", &self.prestige)?;
        state.serialize_field("perks", &self.perks)?;
        let leaders = serialize_array(&self.leaders);
        state.serialize_field("leaders", &leaders)?;
        state.serialize_field("found_date", &self.found_date)?;
        state.end()
    }
}

impl Renderable for Dynasty {

    fn get_context(&self) -> minijinja::Value {
        return context!{dynasty=>self};
    }

    fn get_template() -> &'static str {
        "dynastyTemplate.html"
    }

    fn get_subdir() -> &'static str {
        "dynasties"
    }

    fn render_all(&self, renderer: &mut Renderer) {
        if !renderer.render(self){
            return;
        }
        for leader in self.leaders.iter(){
            leader.get_internal().render_all(renderer);
        }
    }
}

impl Cullable for Dynasty {
    fn set_depth(&mut self, depth:usize, localization:&Localizer) {
        if depth <= self.depth && depth != 0{
            return;
        }
        if !self.name_localized{
            if self.name.is_some() {
                self.name = Some(localization.localize(self.name.as_ref().unwrap().as_str()));
            }
            else{
                self.name = Some(GameString::wrap("Unknown".to_owned()));
            }
            self.name_localized = true;
        }
        if depth == 0 {
            return;
        }
        if !self.localized{
            for perk in self.perks.iter_mut(){
                perk.0 = localization.localize(perk.0.as_str());
            }
            self.localized = true;
        }
        self.depth = depth;
        for leader in self.leaders.iter(){
            let o = leader.try_get_internal_mut();
            if o.is_ok(){
                o.unwrap().set_depth(depth - 1, localization);
            }
        }
        if self.parent.as_ref().is_some(){
            let o = self.parent.as_ref().unwrap().try_get_internal_mut();
            if o.is_ok(){
                o.unwrap().set_depth(depth - 1, localization);
            }
        }
    }

    fn get_depth(&self) -> usize {
        self.depth
    }
}
