use std::{
    ops::{Deref, DerefMut},
    path::Path,
    slice::Iter,
    str::FromStr,
};

use jomini::common::{Date, PdsDate};
use serde::Serialize;

use super::{
    super::{
        display::{Grapher, ProceduralPath, Renderable, TreeNode},
        game_data::{GameData, Localizable, LocalizationError, Localize, MapGenerator, MapImage},
        jinja_env::TITLE_TEMPLATE_NAME,
        parser::{
            GameObjectMap, GameObjectMapping, GameState, ParsingError, SaveFileObject,
            SaveFileValue,
        },
        types::{GameString, Wrapper, WrapperMut},
    },
    Character, Culture, EntityRef, Faith, FromGameObject, GameObjectDerived, GameObjectEntity,
    GameRef,
};

#[derive(Serialize)]
pub struct TitleData {
    key: GameString,
    name: GameString,
    de_jure: Option<GameRef<Title>>,
    de_facto: Option<GameRef<Title>>,
    de_jure_vassals: Vec<GameRef<Title>>,
    de_facto_vassals: Vec<GameRef<Title>>,
    history: Vec<(Date, Option<GameRef<Character>>, GameString)>,
    claims: Vec<GameRef<Character>>,
    capital: Option<GameRef<Title>>,
    color: [u8; 3],
}

impl TitleData {
    fn new(
        key: GameString,
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError> {
        let mut title = Self {
            key: key,
            name: base.get_string("name")?,
            color: base
                .get("color")
                .map(|v| v.as_object().and_then(|obj| obj.as_array()))
                .transpose()?
                .map_or([70, 255, 70], |color_obj| {
                    [
                        color_obj[0].as_integer().unwrap() as u8,
                        color_obj[1].as_integer().unwrap() as u8,
                        color_obj[2].as_integer().unwrap() as u8,
                    ]
                }),
            de_jure: base
                .get("de_jure_liege")
                .map(|liege| {
                    liege
                        .as_id()
                        .and_then(|liege_id| Ok(game_state.get_title(&liege_id)))
                })
                .transpose()?,
            de_facto: base
                .get("de_facto_liege")
                .map(|liege| {
                    liege
                        .as_id()
                        .and_then(|liege_id| Ok(game_state.get_title(&liege_id).clone()))
                })
                .transpose()?,
            de_jure_vassals: Vec::default(),
            de_facto_vassals: Vec::default(),
            history: Vec::default(),
            claims: Vec::default(),
            capital: base
                .get("capital")
                .map(|capital| capital.as_id().and_then(|id| Ok(game_state.get_title(&id))))
                .transpose()?,
        };
        if let Some(claims) = base.get("claim") {
            if let SaveFileValue::Object(claims) = claims {
                for claim in claims.as_array()? {
                    title
                        .claims
                        .push(game_state.get_character(&claim.as_id()?).clone());
                }
            } else {
                title
                    .claims
                    .push(game_state.get_character(&claims.as_id()?).clone());
            }
        }

        if let Some(hist) = base.get("history") {
            for (h, val) in hist.as_object()?.as_map()? {
                let character;
                let action: GameString;
                if let SaveFileValue::Object(o) = val {
                    match o {
                        SaveFileObject::Array(arr) => {
                            for entry in arr {
                                let loc_action;
                                let loc_character;
                                if let SaveFileValue::Object(o) = entry {
                                    let o = o.as_map()?;
                                    loc_action = o.get_string("type")?;
                                    if let Some(holder) = o.get("holder") {
                                        loc_character = Some(
                                            game_state.get_character(&holder.as_id()?).clone(),
                                        );
                                    } else {
                                        loc_character = None;
                                    }
                                } else {
                                    loc_action = GameString::from("Inherited");
                                    loc_character =
                                        Some(game_state.get_character(&entry.as_id()?).clone());
                                }
                                title
                                    .history
                                    .push((Date::from_str(h)?, loc_character, loc_action))
                            }
                            continue; //if it's an array we handled all the adding already in the loop above
                        }
                        SaveFileObject::Map(o) => {
                            action = o.get_string("type")?;
                            match o.get("holder") {
                                Some(h) => {
                                    character = Some(game_state.get_character(&h.as_id()?).clone());
                                }
                                None => {
                                    character = None;
                                }
                            }
                        }
                    }
                } else {
                    action = GameString::from("Inherited");
                    character = Some(game_state.get_character(&val.as_id()?).clone());
                }
                title.history.push((Date::from_str(h)?, character, action));
            }
        }
        //sort history by the first element of the tuple (the date) in descending order
        title.history.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(title)
    }
}

#[derive(Serialize)]
#[serde(tag = "tier")]
pub enum Title {
    Empire(TitleData),
    Kingdom(TitleData),
    Duchy(TitleData),
    County {
        #[serde(flatten)]
        data: TitleData,
        culture: Option<GameRef<Culture>>,
        faith: Option<GameRef<Faith>>,
    },
    Barony(TitleData),
    Other(TitleData),
}

impl Deref for Title {
    type Target = TitleData;

    fn deref(&self) -> &Self::Target {
        match self {
            Title::Empire(data) => data,
            Title::Kingdom(data) => data,
            Title::Duchy(data) => data,
            Title::County { data, .. } => data,
            Title::Barony(data) => data,
            Title::Other(data) => data,
        }
    }
}

impl DerefMut for Title {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Title::Empire(data) => data,
            Title::Kingdom(data) => data,
            Title::Duchy(data) => data,
            Title::County { data, .. } => data,
            Title::Barony(data) => data,
            Title::Other(data) => data,
        }
    }
}

impl Title {
    /// Adds a de jure vassal to the title
    pub fn add_jure_vassal(&mut self, vassal: GameRef<Title>) {
        self.de_jure_vassals.push(vassal);
    }

    /// Adds a de facto vassal to the title
    pub fn add_facto_vassal(&mut self, vassal: GameRef<Title>) {
        self.de_facto_vassals.push(vassal);
    }

    /// Recursively gets all the de facto barony keys of the title
    pub fn get_barony_keys(&self) -> Vec<GameString> {
        if let Title::Barony(_) = self {
            return vec![self.key.clone()];
        } else {
            let mut provinces = Vec::new();
            for v in &self.de_facto_vassals {
                if let Some(v) = v.get_internal().inner() {
                    provinces.append(&mut v.get_barony_keys());
                }
            }
            return provinces;
        }
    }

    pub fn get_de_jure_barony_keys(&self) -> Vec<GameString> {
        if let Title::Barony(_) = self {
            return vec![self.key.clone()];
        } else {
            let mut provinces = Vec::new();
            for v in &self.de_facto_vassals {
                if let Some(v) = v.get_internal().inner() {
                    provinces.append(&mut v.get_de_jure_barony_keys());
                }
            }
            return provinces;
        }
    }

    /// Returns the key of the title
    pub fn get_key(&self) -> GameString {
        self.key.clone()
    }

    /// Returns an iterator over the history of the title
    pub fn get_history_iter(&self) -> Iter<(Date, Option<GameRef<Character>>, GameString)> {
        self.history.iter()
    }

    /// Returns the capital of the title
    pub fn get_capital(&self) -> Option<GameRef<Title>> {
        self.capital.clone()
    }

    /// Returns the holder of the title
    pub fn get_holder(&self) -> Option<GameRef<Character>> {
        if let Some(entry) = self.history.last() {
            return entry.1.clone();
        }
        None
    }

    /// Returns the type of the title
    pub fn get_type(&self) -> Option<&'static str> {
        match self {
            Title::Empire(_) => Some("Empire"),
            Title::Kingdom(_) => Some("Kingdom"),
            Title::Duchy(_) => Some("Duchy"),
            Title::County { .. } => Some("County"),
            Title::Barony(_) => Some("Barony"),
            Title::Other(_) => None,
        }
    }
}

impl FromGameObject for Title {
    fn from_game_object(
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError> {
        let key = base.get_string("key")?;
        let inner = TitleData::new(key.clone(), base, game_state)?;
        Ok(match key.as_ref().chars().next().unwrap() {
            'e' => Self::Empire(inner),
            'k' => Self::Kingdom(inner),
            'd' => Self::Duchy(inner),
            'c' => Self::County {
                data: inner,
                culture: None,
                faith: None,
            },
            'b' => Self::Barony(inner),
            _ => Self::Other(inner),
        })
    }

    fn finalize(&mut self, reference: &GameRef<Title>) {
        if let Some(de_jure) = &self.de_jure {
            if let Some(de_jure) = de_jure.get_internal_mut().inner_mut() {
                de_jure.add_jure_vassal(reference.clone());
            }
        }
        if let Some(de_facto) = &self.de_facto {
            if let Some(de_facto) = de_facto.get_internal_mut().inner_mut() {
                de_facto.add_facto_vassal(reference.clone());
            }
        }
    }
}

impl GameObjectDerived for Title {
    fn get_name(&self) -> GameString {
        self.name.clone()
    }

    fn get_references<E: From<EntityRef>, C: Extend<E>>(&self, collection: &mut C) {
        if let Some(de_jure) = &self.de_jure {
            collection.extend([E::from(de_jure.clone().into())]);
        }
        if let Some(de_facto) = &self.de_facto {
            collection.extend([E::from(de_facto.clone().into())]);
        }
        for v in &self.de_jure_vassals {
            collection.extend([E::from(v.clone().into())]);
        }
        for v in &self.de_facto_vassals {
            collection.extend([E::from(v.clone().into())]);
        }
        for c in &self.claims {
            collection.extend([E::from(c.clone().into())]);
        }
        if let Some(capital) = &self.capital {
            collection.extend([E::from(capital.clone().into())]);
        }
    }
}

impl TreeNode<Vec<GameRef<Title>>> for Title {
    fn get_children(&self) -> Option<Vec<GameRef<Title>>> {
        if self.de_jure_vassals.is_empty() {
            return None;
        }
        Some(self.de_jure_vassals.clone())
    }

    fn get_class(&self) -> Option<GameString> {
        if let Some(tp) = self.get_type() {
            return Some(tp.into());
        }
        None
    }

    fn get_parent(&self) -> Option<Vec<GameRef<Title>>> {
        if let Some(de_jure) = &self.de_jure {
            return Some(vec![de_jure.clone()]);
        }
        None
    }
}

impl ProceduralPath for Title {
    fn get_subdir() -> &'static str {
        "titles"
    }
}

impl Renderable for GameObjectEntity<Title> {
    fn get_template() -> &'static str {
        TITLE_TEMPLATE_NAME
    }

    fn render(&self, path: &Path, game_state: &GameState, _: Option<&Grapher>, data: &GameData) {
        if let Some(map) = data.get_map() {
            if let Some(title) = self.inner() {
                if title.de_facto_vassals.len() == 0 {
                    return;
                }
                let mut buf = path.join(Title::get_subdir());
                buf.push(self.id.to_string() + ".png");
                let mut title_map = map.create_map_flat(title.get_barony_keys(), title.color);
                title_map.draw_text(format!(
                    "{} at {}",
                    title.name,
                    game_state.get_current_date().unwrap().iso_8601()
                ));
                title_map.save_in_thread(&buf);
            }
        }
    }
}

impl Localizable for Title {
    fn localize(&mut self, localization: &GameData) -> Result<(), LocalizationError> {
        if self.name == self.key {
            self.name = localization.localize(&self.key)?;
        }
        //for o in self.history.iter_mut() {
        //    o.2 = localization.localize(o.2.as_str());
        //}
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_title_barony() {
        let title = Title::Barony(TitleData {
            key: "b_test".into(),
            name: "Test".into(),
            de_jure: None,
            de_facto: None,
            de_jure_vassals: Vec::new(),
            de_facto_vassals: Vec::new(),
            history: Vec::new(),
            claims: Vec::new(),
            capital: None,
            color: [70, 255, 70],
        });
        let serialized = serde_json::to_string(&title).unwrap();
        assert_eq!(
            serialized,
            r#"{"tier":"Barony","key":"b_test","name":"Test","de_jure":null,"de_facto":null,"de_jure_vassals":[],"de_facto_vassals":[],"history":[],"claims":[],"capital":null,"color":[70,255,70]}"#
        );
    }

    #[test]
    fn test_serialize_county() {
        let title = Title::County {
            data: TitleData {
                key: "c_test".into(),
                name: "Test".into(),
                de_jure: None,
                de_facto: None,
                de_jure_vassals: Vec::new(),
                de_facto_vassals: Vec::new(),
                history: Vec::new(),
                claims: Vec::new(),
                capital: None,
                color: [70, 255, 70],
            },
            culture: None,
            faith: None,
        };
        let serialized = serde_json::to_string(&title).unwrap();
        assert_eq!(
            serialized,
            r#"{"tier":"County","key":"c_test","name":"Test","de_jure":null,"de_facto":null,"de_jure_vassals":[],"de_facto_vassals":[],"history":[],"claims":[],"capital":null,"color":[70,255,70],"culture":null,"faith":null}"#
        );
    }
}
