use minijinja::{Value, Environment};

/// Create a new [Environment] with the functions initialized.
/// The functions are:
/// - [render_ref](reference:Value, subdir:String) -> String
/// 
/// # Returns
/// A new [Environment] with the functions initialized.
/// The environment is static, so it can be used in multiple threads.
/// Also the environment **doesn't include any templates**
pub fn create_env() -> Environment<'static>{
    let mut env = Environment::new();
    env.add_function("render_ref", render_ref);
    env
}

fn render_ref(reference: Value, subdir:String) -> String{
    let n = reference.get_attr("name").unwrap();
    let name = n.as_str().unwrap();
    //FIXME the second unwrap fails, HOW DO I GET THE VALUE THEN?
    if reference.get_attr("shallow").unwrap().as_str().unwrap() == "True"{
        format!("{}", name)
    }
    else{
        format!("<a href=\"../{}/{}.html\">{}</a>", subdir, reference.get_attr("id").unwrap().as_str().unwrap(), name)
    }
}