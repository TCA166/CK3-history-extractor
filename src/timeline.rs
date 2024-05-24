use minijinja::context;
use serde::Serialize;
use serde::ser::SerializeStruct;

use crate::{game_object::GameString, game_state::GameState, graph::Grapher, localizer::Localizer, map::GameMap, renderer::{Cullable, Renderable, Renderer}, structures::{Character, DerivedRef, GameObjectDerived, Title}, types::Wrapper};

//const CREATED_STR:&str = "Created";
const DESTROYED_STR:&str = "destroyed";
const USURPED_STR:&str = "usurped";
const CONQUERED_START_STR:&str = "conq"; //this should match both 'conquered' and 'conquest holy war'

pub struct Timeline{
    lifespans: Vec<(DerivedRef<Title>, Vec<(u32, u32)>)>,
    latest_event: u32,
    events: Vec<(u32, DerivedRef<Character>, DerivedRef<Title>, GameString)> // (year, character, title, event_type<conquered, usurped, etc.
}

impl Timeline{
    pub fn new(game_state: &GameState) -> Self{
        let mut lifespans = Vec::new();
        let mut latest_event = 0;
        let mut event_checkout = Vec::new();
        for (_, title) in game_state.get_title_iter(){
            //first we handle the empires and collect titles that might be relevant for events
            let t = title.get_internal();
            let k = t.get_key();
            let hist = t.get_history_iter();
            if hist.len() == 0{
                continue;
            }
            if k.is_some(){ //if the key is there
                let kingdom = k.as_ref().unwrap().starts_with("k_");
                if kingdom {
                    event_checkout.push(title.clone());
                    //event_checkout.push(title.get_internal().get_capital().unwrap().clone());
                    continue;
                }
                let empire = k.as_ref().unwrap().starts_with("e_");
                if !empire{
                    continue;
                }
                event_checkout.push(title.clone());
                event_checkout.push(title.get_internal().get_capital().unwrap().clone());
                let mut item = (DerivedRef::from_derived(title.clone()), Vec::new());
                let mut empty = true;
                let mut start = 0;
                //TODO review performance of this
                for entry in hist{
                    let yr = entry.0.split_once('.').unwrap().0.parse().unwrap();
                    if yr > latest_event{
                        latest_event = yr;
                    }
                    let event = entry.2.as_str();
                    if event == DESTROYED_STR { //if it was destroyed we mark the end of the lifespan
                        item.1.push((start, yr));
                        empty = true;
                    }  else if empty { //else if we are not in a lifespan we start a new one
                        start = yr;
                        empty = false;
                    }
                }
                if empire {
                    if !empty{
                        item.1.push((start, 0));
                    }
                    //println!("{} {:?}", title.get_internal().get_key().unwrap(), item.1);
                    lifespans.push(item);
                }
            }
        }
        let mut events:Vec<(u32, DerivedRef<Character>, DerivedRef<Title>, GameString)> = Vec::new();
        for title in event_checkout{
            let tit = title.get_internal();
            let mut hist = tit.get_history_iter();
            let first_char = hist.next().as_ref().unwrap().1.as_ref().unwrap().get_internal();
            let mut faith = first_char.get_faith().unwrap().get_internal().get_id();
            let mut culture = first_char.get_culture().unwrap().get_internal().get_id();
            for entry in hist{
                let char = entry.1.as_ref();
                if char.is_none(){
                    continue;
                }
                let char = char.unwrap();
                let event = entry.2.as_str();
                let ch = char.get_internal();
                let char_faith = ch.get_faith();
                let ch_faith = char_faith.as_ref().unwrap().get_internal();
                let char_culture = ch.get_culture();
                let ch_culture = char_culture.as_ref().unwrap().get_internal();
                if event == USURPED_STR || event.starts_with(CONQUERED_START_STR){
                    let year:u32 = entry.0.split_once('.').unwrap().0.parse().unwrap();
                    if ch_faith.get_id() != faith {
                        let text = format!("claimed for the {} faith", ch_faith.get_name().clone());
                        events.push((year, DerivedRef::from_derived(char.clone()), DerivedRef::from_derived(title.clone()), GameString::wrap(text)));
                        faith = ch_faith.get_id();
                    } else if ch_culture.get_id() != culture {
                        let text = format!("conquered for the {} people", ch_culture.get_name().clone());
                        events.push((year, DerivedRef::from_derived(char.clone()), DerivedRef::from_derived(title.clone()), GameString::wrap(text)));
                        culture = ch_culture.get_id();
                    }
                } else {
                    if ch_faith.get_id() != faith {
                        faith = ch_faith.get_id();
                    }
                    if ch_culture.get_id() != culture {
                        culture = ch_culture.get_id();
                    }
                }
            }
        }
        events.sort_by(|a, b| a.0.cmp(&b.0));
        Self{
            lifespans,
            latest_event,
            events
        }
    }
}

impl Serialize for Timeline{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Timeline", 3)?;
        state.serialize_field("lifespans", &self.lifespans)?;
        state.serialize_field("latest_event", &self.latest_event)?;
        state.serialize_field("events", &self.events)?;
        state.end()
    }
}

impl GameObjectDerived for Timeline{
    fn get_id(&self) -> u32 {
        0
    }

    fn get_name(&self) -> GameString {
        GameString::wrap("Timeline".to_string())
    }
}

impl Cullable for Timeline{
    fn get_depth(&self) -> usize {
        0
    }

    fn is_ok(&self) -> bool {
        true
    }

    fn set_depth(&mut self, depth:usize, localization:&Localizer) {
        for (title, _) in self.lifespans.iter_mut(){
            title.set_depth(depth, localization);
        }
        for (_, char, title, _) in self.events.iter_mut(){
            char.set_depth(depth, localization);
            title.set_depth(depth, localization);
        }
    }
}

impl Renderable for Timeline{
    fn get_context(&self) -> minijinja::Value {
        context!{timeline=>self}
    }

    fn get_subdir() -> &'static str {
        "."
    }
    
    fn get_path(&self, path: &str) -> String {
        format!("{}/timeline.html", path)
    }

    fn get_template() -> &'static str {
        "timelineTemplate.html"
    }

    fn render_all(&self, renderer: &mut Renderer, game_map: Option<&GameMap>, grapher: Option<&Grapher>) {
        if grapher.is_some(){
            let path = format!("{}/timeline.svg", renderer.get_path());
            Grapher::create_timeline_graph(&self.lifespans, &self.events, self.latest_event, &path)
        }
        renderer.render(self);
        for (title, _) in &self.lifespans{
            title.render_all(renderer, game_map, grapher);
        }
    }
}
