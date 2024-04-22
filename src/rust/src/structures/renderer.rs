use minijinja::Environment;

pub trait Renderable{
    fn render(&self, env: &Environment, template_name: &'static String) -> String;
}
