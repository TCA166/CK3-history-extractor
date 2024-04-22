use minijinja::{Environment, context};

pub trait renderable{
    fn render(&self) -> String;
}

impl renderable for GameObject{
    fn render(&self, env: &Environment, templateName: String, templateHook: String) -> String{
        let tpl = env.get_template(templateName).unwrap();
        let mut ctx = context::Context::new();
        ctx.set(templateHook, &self);
        return tpl.render(&ctx).unwrap();
    }
}
