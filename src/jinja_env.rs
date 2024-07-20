use std::{fs, path::Path};

use minijinja::{AutoEscape, Environment, Value};

#[cfg(internal)]
static INT_H_TEMPLATE: &str = include_str!("../templates/homeTemplate.html");
#[cfg(internal)]
static INT_C_TEMPLATE: &str = include_str!("../templates/charTemplate.html");
#[cfg(internal)]
static INT_CUL_TEMPLATE: &str = include_str!("../templates/cultureTemplate.html");
#[cfg(internal)]
static INT_DYN_TEMPLATE: &str = include_str!("../templates/dynastyTemplate.html");
#[cfg(internal)]
static INT_FAITH_TEMPLATE: &str = include_str!("../templates/faithTemplate.html");
#[cfg(internal)]
static INT_TITLE_TEMPLATE: &str = include_str!("../templates/titleTemplate.html");
#[cfg(internal)]
static INT_TIMELINE_TEMPLATE: &str = include_str!("../templates/timelineTemplate.html");

pub const H_TEMPLATE_NAME: &str = "homeTemplate";
pub const C_TEMPLATE_NAME: &str = "charTemplate";
pub const CUL_TEMPLATE_NAME: &str = "cultureTemplate";
pub const DYN_TEMPLATE_NAME: &str = "dynastyTemplate";
pub const FAITH_TEMPLATE_NAME: &str = "faithTemplate";
pub const TITLE_TEMPLATE_NAME: &str = "titleTemplate";
pub const TIMELINE_TEMPLATE_NAME: &str = "timelineTemplate";
const TEMPLATE_NAMES: [&str; 7] = [
    H_TEMPLATE_NAME,
    C_TEMPLATE_NAME,
    CUL_TEMPLATE_NAME,
    DYN_TEMPLATE_NAME,
    FAITH_TEMPLATE_NAME,
    TITLE_TEMPLATE_NAME,
    TIMELINE_TEMPLATE_NAME,
];

/// # Environment creation
///
/// Create a new [Environment] with the filters and templates needed for the project.
/// If the internal flag is set to true, it will use the internal templates, otherwise it will use the templates in the templates folder.
/// If the templates folder does not exist, it will attempt use the internal templates regardless of the setting.
///
/// **This function leaks memory.**
///
/// ## Env specifics
///
/// The environment will have no html escaping.
/// 
/// ### Filters
///
/// The environment will have the following filters:
/// - [render_ref] - renders a reference to another object
/// - [handle_tooltips] - removes tooltips from the text
///
/// ### Globals
///
/// The environment will have the following globals:
/// - map_present - whether the map is present
/// - no_vis - whether the visualizations are disabled
pub fn create_env(internal: bool, map_present: bool, no_vis: bool) -> Environment<'static> {
    let mut env = Environment::new();
    env.add_filter("render_ref", render_ref);
    env.add_filter("handle_tooltips", handle_tooltips);
    env.add_global("map_present", map_present);
    env.add_global("no_vis", no_vis);
    env.set_auto_escape_callback(|arg0: &str| determine_auto_escape(arg0));
    if internal || !Path::new("./templates").exists() {
        #[cfg(internal)]
        {
            env.add_template(H_TEMPLATE_NAME, INT_H_TEMPLATE).unwrap();
            env.add_template(C_TEMPLATE_NAME, INT_C_TEMPLATE).unwrap();
            env.add_template(CUL_TEMPLATE_NAME, INT_CUL_TEMPLATE)
                .unwrap();
            env.add_template(DYN_TEMPLATE_NAME, INT_DYN_TEMPLATE)
                .unwrap();
            env.add_template(FAITH_TEMPLATE_NAME, INT_FAITH_TEMPLATE)
                .unwrap();
            env.add_template(TITLE_TEMPLATE_NAME, INT_TITLE_TEMPLATE)
                .unwrap();
            env.add_template(TIMELINE_TEMPLATE_NAME, INT_TIMELINE_TEMPLATE)
                .unwrap();
        }
        #[cfg(not(internal))]
        {
            panic!("Internal templates requested but not compiled in");
        }
    } else {
        let template_dir = fs::read_dir("templates").unwrap();
        for entry in template_dir {
            let entry = entry.unwrap();
            let file_name = entry.file_name();
            //get the name of the file without the extension
            let name = file_name.to_str().unwrap().splitn(2, '.').next().unwrap();
            //it needs to be a template file
            let i = TEMPLATE_NAMES.iter().position(|&x| x == name);
            if i.is_none() {
                continue;
            }
            // WARNING! LEAKS MEMORY
            let template = Box::new(fs::read_to_string(entry.path()).unwrap());
            env.add_template(TEMPLATE_NAMES[i.unwrap()], template.leak())
                .unwrap();
        }
    }
    env
}

fn determine_auto_escape(_value: &str) -> AutoEscape {
    AutoEscape::None
}

/// A function that renders a reference.
/// May be used in the templates as filter(using [Environment::add_filter]) or function(using [Environment::add_function]) to render a reference to another object.
/// If the reference is shallow, it will render just the name, otherwise render it as a link.
/// The function must be rendered without html escape.
/// Calling this on an undefined reference will fail.
fn render_ref(reference: Value, root: Option<bool>) -> String {
    if reference.is_none() {
        return "none".to_string();
    }
    let n = reference.get_attr("name").unwrap();
    let name = n.as_str().unwrap();
    if reference.get_attr("shallow").unwrap().is_true() {
        format!("{}", name)
    } else {
        let subdir = reference.get_attr("subdir").unwrap();
        if subdir.is_none() {
            return format!(
                "<a href=\"./{}.html\">{}</a>",
                reference.get_attr("id").unwrap(),
                name
            );
        } else if root.is_some() && root.unwrap() {
            format!(
                "<a href=\"{}/{}.html\">{}</a>",
                subdir,
                reference.get_attr("id").unwrap(),
                name
            )
        } else {
            format!(
                "<a href=\"../{}/{}.html\">{}</a>",
                subdir,
                reference.get_attr("id").unwrap(),
                name
            )
        }
    }
}

/// A function that handles tooltips.
/// Removes the tooltips from the text and returns the text without the tooltips.
fn handle_tooltips(text: Value) -> String {
    let text = text.as_str().unwrap();
    let mut result = String::new();
    let mut in_tooltip = false;
    let mut in_tooltip_text = false;
    let mut tooltip_text = String::new();
    for c in text.chars() {
        match c {
            '\x15' => {
                // NAK character precedes a tooltip
                in_tooltip = true;
                in_tooltip_text = false;
                tooltip_text.clear();
            }
            ' ' => {
                if in_tooltip && !in_tooltip_text {
                    in_tooltip_text = true;
                } else {
                    result.push(c);
                }
            }
            '!' => {
                // NAK! character ends a tooltip? I think?
                if in_tooltip {
                    in_tooltip = false;
                    in_tooltip_text = false;
                } else {
                    result.push(c);
                }
            }
            _ => {
                if in_tooltip && !in_tooltip_text {
                    tooltip_text.push(c);
                } else {
                    result.push(c);
                }
            }
        }
    }
    return result;
}
