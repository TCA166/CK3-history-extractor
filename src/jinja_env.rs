use std::{fs, path::Path};

use minijinja::{context, Environment, State, UndefinedBehavior, Value};
use serde::{ser::SerializeStruct, Serialize};

use super::{
    display::ProceduralPath,
    game_data::{GameData, Localize},
    structures::{FromGameObject, GameObjectDerived, GameRef},
    types::Wrapper,
};

#[cfg(feature = "internal")]
mod internal_templates {
    pub const INT_H_TEMPLATE: &str = include_str!("../templates/homeTemplate.html");
    pub const INT_C_TEMPLATE: &str = include_str!("../templates/charTemplate.html");
    pub const INT_CUL_TEMPLATE: &str = include_str!("../templates/cultureTemplate.html");
    pub const INT_DYN_TEMPLATE: &str = include_str!("../templates/dynastyTemplate.html");
    pub const INT_HOUSE_TEMPLATE: &str = include_str!("../templates/houseTemplate.html");
    pub const INT_FAITH_TEMPLATE: &str = include_str!("../templates/faithTemplate.html");
    pub const INT_TITLE_TEMPLATE: &str = include_str!("../templates/titleTemplate.html");
    pub const INT_TIMELINE_TEMPLATE: &str = include_str!("../templates/timelineTemplate.html");
    pub const INT_BASE_TEMPLATE: &str = include_str!("../templates/base.html");
    pub const INT_REF_TEMPLATE: &str = include_str!("../templates/refTemplate.html");
}

pub const H_TEMPLATE_NAME: &str = "homeTemplate";
pub const C_TEMPLATE_NAME: &str = "charTemplate";
pub const CUL_TEMPLATE_NAME: &str = "cultureTemplate";
pub const DYN_TEMPLATE_NAME: &str = "dynastyTemplate";
pub const HOUSE_TEMPLATE_NAME: &str = "houseTemplate";
pub const FAITH_TEMPLATE_NAME: &str = "faithTemplate";
pub const TITLE_TEMPLATE_NAME: &str = "titleTemplate";
pub const TIMELINE_TEMPLATE_NAME: &str = "timelineTemplate";
pub const BASE_TEMPLATE_NAME: &str = "base";
pub const REF_TEMPLATE_NAME: &str = "refTemplate";

const TEMPLATE_NAMES: [&str; 10] = [
    H_TEMPLATE_NAME,
    C_TEMPLATE_NAME,
    CUL_TEMPLATE_NAME,
    DYN_TEMPLATE_NAME,
    HOUSE_TEMPLATE_NAME,
    FAITH_TEMPLATE_NAME,
    TITLE_TEMPLATE_NAME,
    TIMELINE_TEMPLATE_NAME,
    BASE_TEMPLATE_NAME,
    REF_TEMPLATE_NAME,
];

const LOCALIZATION_GLOBAL: &str = "localization";
const LOCALIZATION_FUNC_NAME: &str = "localize";

const DERIVED_REF_NAME_ATTR: &str = "name";
const DERIVED_REF_SUBDIR_ATTR: &str = "subdir";
const DERIVED_REF_ID_ATTR: &str = "id";

impl<T: GameObjectDerived + FromGameObject + ProceduralPath> Serialize for GameRef<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let internal = self.get_internal();
        if let Some(inner) = internal.inner() {
            let mut state = serializer.serialize_struct("DerivedRef", 3)?;
            state.serialize_field(DERIVED_REF_ID_ATTR, &internal.get_id())?;
            state.serialize_field(DERIVED_REF_NAME_ATTR, &inner.get_name())?;
            state.serialize_field(DERIVED_REF_SUBDIR_ATTR, T::get_subdir())?;
            state.end()
        } else {
            serializer.serialize_none()
        }
    }
}

// MAYBE there's a better way of providing localization, however, I have yet to find it

/* What we do here, is allow for all Value objects to act as localizer, and
then embed the localizer in the environment. This is sort of bad. Performance
wise at least */

impl Localize<String> for Value {
    fn lookup<K: AsRef<str>>(&self, key: K) -> Option<String> {
        self.get_attr(key.as_ref())
            .ok()
            .map(|x| x.as_str().and_then(|x| Some(x.to_string())))
            .flatten()
    }

    fn is_empty(&self) -> bool {
        self.is_none()
    }
}

/// # Environment creation
///
/// Create a new [Environment] with the filters and templates needed for the project.
/// If the internal flag is set to true, it will use the internal templates, otherwise it will use the templates in the templates folder.
/// If the templates folder does not exist, it will attempt use the internal templates regardless of the setting.
///
/// ## Env specifics
///
/// The environment will have no html escaping, and will not permit undefined chicanery.
///
/// ### Filters
///
/// The environment will have the following filters:
/// - [render_ref] - renders a reference to another object
/// - [localize] - localizes the provided string
///
/// ### Globals
///
/// The environment will have the following globals:
/// - map_present - whether the map is present
/// - no_vis - whether the visualizations are disabled
///
pub fn create_env<'a>(
    internal: bool,
    map_present: bool,
    no_vis: bool,
    data: &GameData,
) -> Environment<'a> {
    let mut env = Environment::new();
    env.set_lstrip_blocks(true);
    env.set_trim_blocks(true);
    env.add_filter("render_ref", render_ref);
    env.add_filter(LOCALIZATION_FUNC_NAME, localize);
    env.add_function(LOCALIZATION_FUNC_NAME, localize);
    env.add_global("map_present", map_present);
    env.add_global("no_vis", no_vis);
    env.add_global(
        LOCALIZATION_GLOBAL,
        Value::from_serialize(data.get_localizer()),
    );
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    let template_path = Path::new("./templates");
    if internal || !template_path.exists() {
        #[cfg(feature = "internal")]
        {
            use internal_templates::*;
            env.add_template(H_TEMPLATE_NAME, INT_H_TEMPLATE).unwrap();
            env.add_template(C_TEMPLATE_NAME, INT_C_TEMPLATE).unwrap();
            env.add_template(CUL_TEMPLATE_NAME, INT_CUL_TEMPLATE)
                .unwrap();
            env.add_template(DYN_TEMPLATE_NAME, INT_DYN_TEMPLATE)
                .unwrap();
            env.add_template(HOUSE_TEMPLATE_NAME, INT_HOUSE_TEMPLATE)
                .unwrap();
            env.add_template(FAITH_TEMPLATE_NAME, INT_FAITH_TEMPLATE)
                .unwrap();
            env.add_template(TITLE_TEMPLATE_NAME, INT_TITLE_TEMPLATE)
                .unwrap();
            env.add_template(TIMELINE_TEMPLATE_NAME, INT_TIMELINE_TEMPLATE)
                .unwrap();
            env.add_template(BASE_TEMPLATE_NAME, INT_BASE_TEMPLATE)
                .unwrap();
            env.add_template(REF_TEMPLATE_NAME, INT_REF_TEMPLATE)
                .unwrap();
        }
        #[cfg(not(feature = "internal"))]
        {
            panic!("Internal templates requested but not compiled in");
        }
    } else {
        let template_dir = fs::read_dir(template_path).unwrap();
        for read_result in template_dir {
            match read_result {
                Ok(entry) => {
                    //it needs to be a template file
                    let path = entry.path();
                    if !path.is_file() {
                        continue;
                    }
                    let name = TEMPLATE_NAMES
                        .iter()
                        .find(|&x| x == &path.file_stem().unwrap());
                    if let Some(name) = name {
                        env.add_template_owned(*name, fs::read_to_string(path).unwrap())
                            .unwrap();
                    }
                }
                Err(e) => eprintln!("Error reading template directory: {}", e),
            }
        }
    }
    env
}

/// A function that renders a reference.
/// May be used in the templates as filter(using [Environment::add_filter]) or function(using [Environment::add_function]) to render a reference to another object.
/// If the reference is shallow, it will render just the name, otherwise render it as a link.
/// The function must be rendered without html escape.
/// Calling this on an undefined reference will fail.
fn render_ref(state: &State, reference: Value, root: Option<bool>) -> String {
    if reference.is_none() {
        return "".to_string();
    }
    if let Some(name) = reference
        .get_attr(DERIVED_REF_NAME_ATTR)
        .expect("Reference doesn't have attributes")
        .as_str()
    {
        let subdir = reference.get_attr(DERIVED_REF_SUBDIR_ATTR).unwrap();
        let id = reference.get_attr(DERIVED_REF_ID_ATTR).unwrap();
        if state
            .lookup("depth_map")
            .unwrap()
            .get_item(&subdir)
            .unwrap()
            .get_item(&id)
            .unwrap()
            .as_i64()
            .unwrap_or(0)
            <= 0
        {
            format!("{}", name)
        } else {
            state
                .env()
                .get_template(REF_TEMPLATE_NAME)
                .unwrap()
                .render(context! {root=>root, ..reference})
                .unwrap()
        }
    } else {
        "".to_string()
    }
}

fn localize(state: &State, key: &str, value: Option<&str>, provider: Option<&str>) -> String {
    let localizer = state.lookup(LOCALIZATION_GLOBAL).unwrap();
    if let Some(value) = value {
        if let Some(provider) = provider {
            localizer.localize_provider(key, provider, value).unwrap()
        } else {
            localizer
                .localize_query(key, |_| Some(value.to_string()))
                .unwrap()
        }
    } else {
        localizer.localize(key).unwrap()
    }
}
