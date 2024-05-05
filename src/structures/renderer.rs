use minijinja::Environment;

use super::GameObjectDerived;

/// Trait for objects that can be rendered into a html page.
/// Since this uses [minijinja] the [serde::Serialize] trait is also needed.
/// Each object that implements this trait should have a corresponding template file in the templates folder.
pub trait Renderable: GameObjectDerived{
    /// Renders the object into a html string.
    fn render(&self, env: &Environment) -> Option<String>;

    /// Returns the subdirectory name where the object should be written to.
    fn get_subdir(&self) -> &'static str;

    /// Returns the path where the object should be written to.
    fn get_path(&self, path: &str) -> String{
        format!("{}/{}/{}.html", path, self.get_subdir(), self.get_id())
    }

    /// Returns true if the object has already been rendered.
    fn is_rendered(&self, path: &str) -> bool{
        //check if the file exists
        std::path::Path::new(&self.get_path(path)).exists()
    }

    /// Renders the object into a html string and writes it to a file.
    fn render_to_file(&self, env: &Environment, path: &str) -> std::io::Result<()>{
        if self.is_rendered(path){
            return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "File already exists"))
        }
        let rendered = self.render(env);
        if rendered.is_none(){
            return Ok(())
        }
        std::fs::write(self.get_path(path), rendered.unwrap())
    }

    /// Renders all objects in the tree and writes them to files.
    fn render_all(&self, env: &Environment, path: &str) -> std::io::Result<()>;
}

/// Trait for objects that can be culled.
/// This is used to limit object serialization to a certain depth.
/// Not all [Renderable] objects need to implement this trait.
pub trait Cullable{
    /// Set the depth of the object.
    /// Ideally this should be called on the root object once and the depth should be propagated to all children.
    /// Also ideally should do nothing if the depth is less than or equal to the current depth.
    fn set_depth(&mut self, depth:usize);

    /// Get the depth of the object.
    fn get_depth(&self) -> usize;
}
