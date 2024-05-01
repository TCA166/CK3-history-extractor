use minijinja::Environment;

/// Trait for objects that can be rendered into a html page.
/// Since this uses [minijinja] the [serde::Serialize] trait is also needed.
/// Each object that implements this trait should have a corresponding template file in the templates folder.
pub trait Renderable{
    fn render(&self, env: &Environment) -> String;
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
