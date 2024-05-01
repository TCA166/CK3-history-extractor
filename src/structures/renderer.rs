use minijinja::Environment;

/// Trait for objects that can be rendered into a html page.
pub trait Renderable{
    fn render(&self, env: &Environment) -> String;
}
