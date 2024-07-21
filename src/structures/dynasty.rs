use serde::{ser::SerializeStruct, Serialize};

use super::super::{
    display::{Cullable, Localizer, Renderable, RenderableType, Renderer},
    game_object::{GameObject, GameString, SaveFileValue},
    game_state::GameState,
    jinja_env::DYN_TEMPLATE_NAME,
    types::{Wrapper, WrapperMut},
};
use super::{
    serialize_array, Character, Culture, DerivedRef, DummyInit, Faith, GameId, GameObjectDerived,
    Shared,
};

pub struct Dynasty {
    id: GameId,
    parent: Option<Shared<Dynasty>>,
    name: Option<GameString>,
    members: u32,
    member_list: Vec<Shared<Character>>,
    houses: u32,
    prestige_tot: f32,
    prestige: f32,
    perks: Vec<(GameString, u8)>,
    leaders: Vec<Shared<Character>>,
    found_date: Option<GameString>,
    motto: Option<(GameString, Vec<GameString>)>,
    depth: usize,
    localized: bool,
    name_localized: bool,
}

impl Dynasty {
    /// Gets the faith of the dynasty.
    /// Really this is just the faith of the current house leader.
    pub fn get_faith(&self) -> Option<Shared<Faith>> {
        if self.leaders.is_empty() {
            return self.member_list.last().unwrap().get_internal().get_faith();
        }
        self.leaders.last().unwrap().get_internal().get_faith()
    }

    /// Gets the culture of the dynasty.
    /// Really this is just the culture of the current house leader.
    pub fn get_culture(&self) -> Option<Shared<Culture>> {
        if self.leaders.is_empty() {
            return self
                .member_list
                .last()
                .unwrap()
                .get_internal()
                .get_culture();
        }
        self.leaders.last().unwrap().get_internal().get_culture()
    }

    /// Registers a new house in the dynasty
    pub fn register_house(&mut self) {
        self.houses += 1;
    }

    /// Registers a new member in the dynasty
    pub fn register_member(&mut self, member: Shared<Character>) {
        self.members += 1;
        self.member_list.push(member.clone());
        if self.parent.as_ref().is_some() {
            let mut p = self.parent.as_ref().unwrap().try_get_internal_mut();
            if p.is_ok() {
                p.as_mut().unwrap().register_member(member);
            }
        }
    }

    pub fn get_founder(&self) -> Shared<Character> {
        if self.leaders.is_empty() {
            return self.member_list.first().unwrap().clone();
        }
        self.leaders.first().unwrap().clone()
    }
}

///Gets the perks of the dynasty and appends them to the perks vector
fn get_perks(perks: &mut Vec<(GameString, u8)>, base: &GameObject) {
    let perks_obj = base.get("perk");
    if perks_obj.is_some() {
        for p in perks_obj.unwrap().as_object().unwrap().get_array_iter() {
            let perk = p.as_string();
            //get the split perk by the second underscore
            let mut i: u8 = 0;
            let mut key: Option<&str> = None;
            let mut val: u8 = 0;
            for el in perk.rsplitn(2, '_') {
                if i == 0 {
                    val = el.parse::<u8>().unwrap();
                } else {
                    key = Some(el);
                }
                i += 1;
            }
            if key.is_none() {
                continue;
            }
            let mut added = false;
            for perk in perks.iter_mut() {
                if perk.0.as_str() == key.unwrap() {
                    if perk.1 < val {
                        perk.1 = val;
                    }
                    added = true;
                    break;
                }
            }
            if added {
                continue;
            }
            //if the perk is not found, add it
            perks.push((GameString::wrap(key.unwrap().to_owned()), val));
        }
    }
}

///Gets the leaders of the dynasty and appends them to the leaders vector
fn get_leaders(
    leaders: &mut Vec<Shared<Character>>,
    base: &GameObject,
    game_state: &mut GameState,
) {
    let leaders_obj = base.get("historical");
    if leaders_obj.is_some() {
        if !leaders.is_empty() {
            leaders.clear();
        }
        for l in leaders_obj.unwrap().as_object().unwrap().get_array_iter() {
            leaders.push(game_state.get_character(&l.as_id()).clone());
        }
    } else if leaders.is_empty() {
        let current = base.get("dynasty_head");
        if current.is_some() {
            leaders.push(game_state.get_character(&current.unwrap().as_id()));
        } else {
            let current = base.get("head_of_house");
            if current.is_some() {
                leaders.push(game_state.get_character(&current.unwrap().as_id()));
            }
        }
    }
}

///Gets the prestige of the dynasty and returns a tuple with the total prestige and the current prestige
fn get_prestige(base: &GameObject) -> (f32, f32) {
    let currency = base.get("prestige");
    let mut prestige_tot = 0.0;
    let mut prestige = 0.0;
    if currency.is_some() {
        let o = currency.unwrap().as_object().unwrap();
        match o.get("accumulated").unwrap() {
            SaveFileValue::Object(ref o) => {
                prestige_tot = o.get_string_ref("value").parse::<f32>().unwrap();
            }
            SaveFileValue::String(ref o) => {
                prestige_tot = o.parse::<f32>().unwrap();
            }
        }
        match o.get("currency") {
            Some(v) => match v {
                SaveFileValue::Object(ref o) => {
                    prestige = o.get_string_ref("value").parse::<f32>().unwrap();
                }
                SaveFileValue::String(ref o) => {
                    prestige = o.parse::<f32>().unwrap();
                }
            },
            None => {}
        }
    }
    (prestige_tot, prestige)
}

///Gets the parent dynasty of the dynasty
fn get_parent(base: &GameObject, game_state: &mut GameState) -> Option<Shared<Dynasty>> {
    let parent_id = base.get("dynasty");
    match parent_id {
        None => None,
        k => {
            let p = game_state.get_dynasty(&k.unwrap().as_id()).clone();
            let m = p.try_get_internal_mut();
            if m.is_err() {
                return None;
            }
            m.unwrap().register_house();
            Some(p)
        }
    }
}

fn get_name(base: &GameObject, parent: Option<Shared<Dynasty>>) -> Option<GameString> {
    let mut n = base.get("name");
    if n.is_none() {
        n = base.get("localized_name");
        if n.is_none() {
            if parent.is_none() {
                return None;
            }
            let p = parent.as_ref().unwrap().get_internal();
            if p.name.is_none() {
                return None;
            }
            //this may happen for dynasties with a house with the same name
            return Some(p.name.as_ref().unwrap().clone());
        }
    }
    Some(n.unwrap().as_string())
}

fn get_date(base: &GameObject) -> Option<GameString> {
    let date = base.get("found_date");
    if date.is_none() {
        return None;
    }
    Some(date.unwrap().as_string())
}

impl DummyInit for Dynasty {
    fn dummy(id: GameId) -> Self {
        Dynasty {
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
            localized: false,
            motto: None,
            name_localized: false,
            member_list: Vec::new(),
        }
    }

    fn init(&mut self, base: &GameObject, game_state: &mut GameState) {
        //NOTE: dynasties can have their main house have the same id
        get_perks(&mut self.perks, &base);
        get_leaders(&mut self.leaders, &base, game_state);
        let res = get_prestige(&base);
        if self.prestige_tot == 0.0 {
            self.prestige_tot = res.0;
            self.prestige = res.1;
        }
        self.parent = get_parent(&base, game_state);
        let name = get_name(&base, self.parent.clone());
        if self.name.is_none() {
            self.name = name;
        }
        self.found_date = get_date(&base);
        let motto_node = base.get("motto");
        if motto_node.is_some() {
            match motto_node.unwrap() {
                SaveFileValue::String(ref s) => {
                    self.motto = Some((s.clone(), Vec::new()));
                }
                SaveFileValue::Object(ref o) => {
                    let key = o.get_string_ref("key");
                    let variables = o.get("variables");
                    let mut vars = Vec::new();
                    for v in variables.unwrap().as_object().unwrap().get_array_iter() {
                        let v = v.as_object().unwrap();
                        let value = v.get_string_ref("value");
                        vars.push(value.clone());
                    }
                    self.motto = Some((key.clone(), vars));
                }
            }
        }
    }
}

impl GameObjectDerived for Dynasty {
    fn get_id(&self) -> GameId {
        self.id
    }

    fn get_name(&self) -> GameString {
        if self.name.is_none() {
            if self.parent.as_ref().is_some() {
                return self.parent.as_ref().unwrap().get_internal().get_name();
            }
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
        let mut state = serializer.serialize_struct("Dynasty", 11)?;
        state.serialize_field("id", &self.id)?;
        if self.parent.as_ref().is_some() {
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
        if self.motto.is_some() {
            let motto_raw = self.motto.as_ref().unwrap();
            let motto = motto_raw.0.split(' ').collect::<Vec<&str>>();
            let var_len = motto_raw.1.len();
            let rebuilt = if var_len == 0 {
                motto_raw.0.clone()
            } else {
                let mut rebuilt = Vec::new();
                let mut j = 0;
                for part in motto {
                    if part.is_empty() || part == "," {
                        rebuilt.push(motto_raw.1[j].as_str());
                        j += 1;
                        if j >= var_len {
                            j = 0; //TODO why can this happen? `(" Through ", ["Safety"])`
                        }
                    } else {
                        rebuilt.push(part);
                    }
                }
                GameString::wrap(rebuilt.join(" "))
            };
            state.serialize_field("motto", &rebuilt)?;
        }
        state.end()
    }
}

impl Renderable for Dynasty {
    fn get_template() -> &'static str {
        DYN_TEMPLATE_NAME
    }

    fn get_subdir() -> &'static str {
        "dynasties"
    }

    fn render_all(&self, stack: &mut Vec<RenderableType>, renderer: &mut Renderer) {
        if !renderer.render(self) {
            return;
        }
        let grapher = renderer.get_grapher();
        if grapher.is_some() {
            let g = grapher.unwrap();
            let path = format!(
                "{}/{}/{}.svg",
                renderer.get_path(),
                Self::get_subdir(),
                self.id
            );
            g.create_dynasty_graph(self, &path);
        }
        for leader in self.leaders.iter() {
            stack.push(RenderableType::Character(leader.clone()));
        }
        if self.parent.as_ref().is_some() {
            stack.push(RenderableType::Dynasty(
                self.parent.as_ref().unwrap().clone(),
            ));
        }
    }
}

impl Cullable for Dynasty {
    fn set_depth(&mut self, depth: usize, localization: &Localizer) {
        if depth <= self.depth && depth != 0 {
            return;
        }
        if !self.name_localized {
            if self.name.is_some() {
                self.name = Some(localization.localize(self.name.as_ref().unwrap().as_str()));
            } else {
                self.name = Some(GameString::wrap("Unknown".to_owned()));
            }
            self.name_localized = true;
        }
        if depth == 0 {
            return;
        }
        if !self.localized {
            for perk in self.perks.iter_mut() {
                perk.0 = localization.localize(perk.0.as_str());
            }
            self.localized = true;
        }
        if self.motto.is_some() {
            let m = self.motto.as_mut().unwrap();
            m.0 = localization.localize(m.0.as_str());
            for v in m.1.iter_mut() {
                *v = localization.localize(v.as_str());
            }
        }
        self.depth = depth;
        for leader in self.leaders.iter() {
            let o = leader.try_get_internal_mut();
            if o.is_ok() {
                o.unwrap().set_depth(depth - 1, localization);
            }
        }
        if self.parent.as_ref().is_some() {
            let o = self.parent.as_ref().unwrap().try_get_internal_mut();
            if o.is_ok() {
                o.unwrap().set_depth(depth - 1, localization);
            }
        }
        for member in self.member_list.iter() {
            let o = member.try_get_internal_mut();
            if o.is_ok() {
                o.unwrap().set_depth(1, localization); //this will localize the character names
            }
        }
    }

    fn get_depth(&self) -> usize {
        self.depth
    }
}
