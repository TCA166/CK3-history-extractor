use minijinja::context;
use serde::Serialize;
use serde::ser::SerializeStruct;

use crate::{game_object::GameString, game_state::GameState, graph::Grapher, localizer::Localizer, map::GameMap, renderer::{Cullable, Renderable, Renderer}, structures::{Character, Faith, GameObjectDerived, Title}, types::{Shared, Wrapper, WrapperMut}};

//const CREATED_STR:&str = "Created";
const DESTROYED_STR:&str = "destroyed";
const USURPED_STR:&str = "usurped";
const CONQUERED_START_STR:&str = "conq"; //this should match both 'conquered' and 'conquest holy war'

pub struct Timeline{
    lifespans: Vec<(Shared<Title>, Vec<(u32, u32)>)>,
    latest_event: u32,
    events: Vec<(u32, Shared<Character>, Shared<Title>, GameString)> // (year, character, title, event_type<conquered, usurped, etc.
}

impl Timeline{
    pub fn new(game_state: &GameState) -> Self{
        let mut lifespans = Vec::new();
        let mut latest_event = 0;
        let mut events:Vec<(u32, Shared<Character>, Shared<Title>, GameString)> = Vec::new();
        for (_, title) in game_state.get_title_iter(){
            let t = title.get_internal();
            let k = t.get_key();
            if k.is_some(){
                //TODO add checking for capitals
                //we collect events and lifespans for empires
                //for kingdoms we only collect events
                let kingdom = k.as_ref().unwrap().starts_with("k_");
                let empire = k.as_ref().unwrap().starts_with("e_");
                if !kingdom && !empire{
                    continue;
                }
                let hist = t.get_history_iter();
                if hist.len() == 0{
                    continue;
                }
                let mut item = (title.clone(), Vec::new());
                let mut empty = true;
                let mut start = 0;
                let mut old_faith: Option<Shared<Faith>> = None;
                //TODO review performance of this
                for entry in hist{
                    let yr = entry.0.split_once('.').unwrap().0.parse().unwrap();
                    if yr > latest_event{
                        latest_event = yr;
                    }
                    let event = entry.2.as_str();
                    if event == DESTROYED_STR && empire{ //if it was destroyed we mark the end of the lifespan
                        item.1.push((start, yr));
                        empty = true;
                    } else if event == USURPED_STR || event.starts_with(CONQUERED_START_STR) && (kingdom || empire) { // if there was a sign of turmoil
                        let faith = entry.1.as_ref().unwrap().get_internal().get_faith().unwrap();
                        if old_faith.is_some() && old_faith.as_ref().unwrap().get_internal().get_id() != faith.get_internal().get_id(){ //if the faith changed we mark the end of the lifespan
                            let action = format!("reclaimed in the name of {}", faith.get_internal().get_name().as_str());
                            events.push((yr, entry.1.as_ref().unwrap().clone(), title.clone(), GameString::wrap(action)));
                        }
                        old_faith = Some(faith);
                    } else if empty && empire{ //else if we are not in a lifespan we start a new one
                        old_faith = entry.1.as_ref().unwrap().get_internal().get_faith();
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
        let mut state = serializer.serialize_struct("Timeline", 2)?;
        state.serialize_field("lifespans", &self.lifespans)?;
        state.serialize_field("latest_event", &self.latest_event)?;
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
        for (title, _) in &self.lifespans{
            title.get_internal_mut().set_depth(depth, localization);
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
            title.get_internal().render_all(renderer, game_map, grapher);
        }
    }
}
