/// A submodule that provides [Renderable] trait for objects that can be rendered.
mod renderer;
pub use renderer::{GetPath, ProceduralPath, Renderable, Renderer};

/// The graphing submodule that handles the creation of graphs from the game state.
mod graph;
pub use graph::{Grapher, TreeNode};

/// A submodule handling the rendering of the timeline page
mod timeline;
pub use timeline::{RealmDifference, Timeline};
