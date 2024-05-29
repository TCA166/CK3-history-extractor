use std::{fs, path::Path};

use minijinja::{AutoEscape, Environment, Value};

#[cfg(internal)]
static INT_H_TEMPLATE:&str = include_str!("../templates/homeTemplate.html");
#[cfg(internal)]
static INT_C_TEMPLATE:&str = include_str!("../templates/charTemplate.html");
#[cfg(internal)]
static INT_CUL_TEMPLATE:&str = include_str!("../templates/cultureTemplate.html");
#[cfg(internal)]
static INT_DYN_TEMPLATE:&str = include_str!("../templates/dynastyTemplate.html");
#[cfg(internal)]
static INT_FAITH_TEMPLATE:&str = include_str!("../templates/faithTemplate.html");
#[cfg(internal)]
static INT_TITLE_TEMPLATE:&str = include_str!("../templates/titleTemplate.html");
#[cfg(internal)]
static INT_TIMELINE_TEMPLATE:&str = include_str!("../templates/timelineTemplate.html");

/// Create a new [Environment] with the filters and templates needed for the project.
/// If the internal flag is set to true, it will use the internal templates, otherwise it will use the templates in the templates folder.
/// If the templates folder does not exist, it will attempt use the internal templates regardless of the setting.
/// The environment will have the following filters:
/// - [render_ref] - renders a reference to another object
pub fn create_env(internal:bool, map_present:bool, no_vis:bool) -> Environment<'static>{
    let mut env = Environment::new();
    env.add_filter("render_ref", render_ref);
    env.add_global("map_present", map_present);
    env.add_global("no_vis", no_vis);
    env.set_auto_escape_callback(|arg0: &str| determine_auto_escape(arg0));
    if internal || !Path::new("./templates").exists(){
        #[cfg(internal)]
        {
            env.add_template("homeTemplate.html", INT_H_TEMPLATE).unwrap();
            env.add_template("charTemplate.html", INT_C_TEMPLATE).unwrap();
            env.add_template("cultureTemplate.html", INT_CUL_TEMPLATE).unwrap();
            env.add_template("dynastyTemplate.html", INT_DYN_TEMPLATE).unwrap();
            env.add_template("faithTemplate.html", INT_FAITH_TEMPLATE).unwrap();
            env.add_template("titleTemplate.html", INT_TITLE_TEMPLATE).unwrap();
            env.add_template("timelineTemplate.html", INT_TIMELINE_TEMPLATE).unwrap();
        }
        #[cfg(not(internal))]
        {
            panic!("Internal templates requested but not compiled in");
        }
    }
    else {
        // LEAKS MEMORY
        let h_template = Box::new(fs::read_to_string("templates/homeTemplate.html").unwrap());
        env.add_template("homeTemplate.html", h_template.leak()).unwrap();
        let c_template = Box::new(fs::read_to_string("templates/charTemplate.html").unwrap());
        env.add_template("charTemplate.html", c_template.leak()).unwrap();
        let cul_template = Box::new(fs::read_to_string("templates/cultureTemplate.html").unwrap());
        env.add_template("cultureTemplate.html", cul_template.leak()).unwrap();
        let dyn_template = Box::new(fs::read_to_string("templates/dynastyTemplate.html").unwrap());
        env.add_template("dynastyTemplate.html", dyn_template.leak()).unwrap();
        let faith_template = Box::new(fs::read_to_string("templates/faithTemplate.html").unwrap());
        env.add_template("faithTemplate.html", faith_template.leak()).unwrap();
        let title_template = Box::new(fs::read_to_string("templates/titleTemplate.html").unwrap());
        env.add_template("titleTemplate.html", title_template.leak()).unwrap();
        let timeline_template = Box::new(fs::read_to_string("templates/timelineTemplate.html").unwrap());
        env.add_template("timelineTemplate.html", timeline_template.leak()).unwrap();
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
fn render_ref(reference: Value, root:Option<bool>) -> String{
    if reference.is_none() {
        return "none".to_string();
    }
    let n = reference.get_attr("name").unwrap();
    let name = n.as_str().unwrap();
    if reference.get_attr("shallow").unwrap().is_true(){
        format!("{}", name)
    }
    else{
        let subdir = reference.get_attr("subdir").unwrap();
        if subdir.is_none() {
            return format!("<a href=\"./{}.html\">{}</a>", reference.get_attr("id").unwrap(), name);
        } else if root.is_some() && root.unwrap(){
            format!("<a href=\"{}/{}.html\">{}</a>", subdir, reference.get_attr("id").unwrap(), name)
        }
        else{
            format!("<a href=\"../{}/{}.html\">{}</a>", subdir, reference.get_attr("id").unwrap(), name)
        }
    }
}
