use minijinja::{AutoEscape, Environment, Value};

/// Create a new [Environment] with the features initialized.
/// All escapes are disabled.
/// 
/// The filters are:
/// - [render_ref](reference:Value, subdir:String) -> String
/// 
/// # Returns
/// A new [Environment] with the functions initialized.
/// The environment is static, so it can be used in multiple threads.
/// Also the environment **doesn't include any templates**
pub fn create_env() -> Environment<'static>{
    let mut env = Environment::new();
    env.add_filter("render_ref", render_ref);
    env.set_auto_escape_callback(|arg0: &str| determine_auto_escape(arg0));
    env
}

fn determine_auto_escape(_value: &str) -> AutoEscape {
    AutoEscape::None
}

/// A function that renders a reference.
/// May be used in the templates as filter(using [Environment::add_filter]) or function(using [Environment::add_function]) to render a reference to another object.
/// If the reference is shallow, it will render just the name, otherwise render it as a link.
/// The function must be rendered without html escape.
fn render_ref(reference: Value, subdir:Option<String>) -> String{
    if reference.is_none() {
        return "none".to_string();
    }
    let n = reference.get_attr("name").unwrap();
    let name = n.as_str().unwrap();
    if reference.get_attr("shallow").unwrap().is_true(){
        format!("{}", name)
    }
    else{
        if subdir.is_none() {
            return format!("<a href=\"./{}.html\">{}</a>", reference.get_attr("id").unwrap(), name);
        }
        format!("<a href=\"../{}/{}.html\">{}</a>", subdir.unwrap(), reference.get_attr("id").unwrap(), name)
    }
}