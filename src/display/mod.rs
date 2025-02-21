/// A submodule that provides [Renderable] and [Cullable] traits for objects that can be rendered.
mod renderer;
pub use renderer::{Cullable, Renderable, Renderer};

/// The graphing submodule that handles the creation of graphs from the game state.
mod graph;
pub use graph::{Grapher, TreeNode};

/// A submodule handling the rendering of the timeline page
mod timeline;
pub use timeline::{RealmDifference, Timeline};

/// A submodule that provides the [RenderableType] enum.
mod renderable_type;
pub use renderable_type::RenderableType;
