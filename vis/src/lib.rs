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

use std::io;
use piet_common::{RenderContext, kurbo::Shape};

/// Only `Shape`s may be `vis`ed, only `Scene` may be `render`ed.
pub trait Vis<R: RenderContext>: Shape {
    fn vis(&self, style_id: Option<StyleId>, theme: &Theme, rc: &mut R);
}

pub trait WriteSvg {
    fn write_svg<W: io::Write>(&self, svg: &mut W) -> io::Result<()>;
}

pub trait WriteSvgWithName {
    fn write_svg_with_name<W: io::Write, S: AsRef<str>>(
        &self,
        svg: &mut W,
        name: S,
    ) -> io::Result<()>;
}

#[derive(Clone, Copy, Debug)]
pub struct PrimId(pub usize);

#[derive(Clone, Copy, Debug)]
pub struct GroupId(pub usize);

#[derive(Clone, Copy, Debug)]
pub struct StyleId(pub usize);
