mod prim;
mod group;
mod style;
mod theme;
mod scene;

pub use prim::Prim;
pub use group::Group;
pub use style::{Style, Stroke, Fill, GradSpec};
pub use scene::Scene;
pub use theme::Theme;

use piet_common::{RenderContext, kurbo::Shape};

/// Only `Shape`s may be `vis`ed, only `Scene`s may be `render`ed.
pub trait Vis<R: RenderContext>: Shape {
    fn vis(&self, style_id: Option<StyleId>, theme: &Theme, rc: &mut R);
}

#[derive(Clone, Copy, Debug)]
pub struct PrimId(pub usize);

#[derive(Clone, Copy, Debug)]
pub struct GroupId(pub usize);

#[derive(Clone, Copy, Debug)]
pub struct StyleId(pub usize);
