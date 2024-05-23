use minijinja::context;
use serde::Serialize;
use serde::ser::SerializeStruct;

use crate::{game_object::GameString, game_state::GameState, graph::Grapher, localizer::Localizer, map::GameMap, renderer::{Cullable, Renderable, Renderer}, structures::{GameObjectDerived, Title}, types::{Shared, Wrapper, WrapperMut}};

//const CREATED_STR:&str = "Created";
const DESTROYED_STR:&str = "destroyed";

pub struct Timeline{
    lifespans: Vec<(Shared<Title>, Vec<(u32, u32)>)>,
    latest_event: u32
}

impl Timeline{
    pub fn new(game_state: &GameState) -> Self{
        let mut lifespans = Vec::new();
        let mut latest_event = 0;
        for (_, title) in game_state.get_title_iter(){
            let t = title.get_internal();
            let k = t.get_key();
            if k.is_some() && k.unwrap().starts_with("e_"){
                let hist = t.get_history_iter();
                if hist.len() == 0{
                    continue;
                }
                let mut item = (title.clone(), Vec::new());
                let mut empty = true;
                let mut start = 0;
                for entry in hist{
                    let yr = entry.0.split_once('.').unwrap().0.parse().unwrap();
                    if yr > latest_event{
                        latest_event = yr;
                    }
                    if entry.2.as_str() == DESTROYED_STR{
                        item.1.push((start, yr));
                        empty = true;
                    } else if empty{
                        start = yr;
                        empty = false;
                    }
                }
                if !empty{
                    item.1.push((start, 0));
                }
                //println!("{} {:?}", title.get_internal().get_key().unwrap(), item.1);
                lifespans.push(item);
            }
        }
        Self{
            lifespans,
            latest_event
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
            Grapher::create_timeline_graph(&self.lifespans, Vec::new(), self.latest_event, &path)
        }
        renderer.render(self);
        for (title, _) in &self.lifespans{
            title.get_internal().render_all(renderer, game_map, grapher);
        }
    }
}
