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
use piet_common::{
    RenderContext,
    kurbo::{TranslateScale, Rect},
};

/// Only `Shape`s may be `vis`ed, only `Scene` may be `render`ed.
pub trait Vis {
    fn bbox(&self, ts: TranslateScale) -> Rect;

    fn vis<R: RenderContext>(
        &self,
        rc: &mut R,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    );

    fn write_svg_with_style<W: io::Write>(
        &self,
        svg: W,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> io::Result<()>;
}

pub trait WriteSvg {
    fn write_svg<W: io::Write>(&self, svg: W) -> io::Result<()>;
}

pub trait WriteSvgWithName {
    fn write_svg_with_name<W: io::Write, S: AsRef<str>>(&self, svg: W, name: S) -> io::Result<()>;
}

#[derive(Clone, Copy, Debug)]
pub struct PrimId(pub usize);

#[derive(Clone, Copy, Debug)]
pub struct GroupId(pub usize);

#[derive(Clone, Copy, Debug)]
pub struct StyleId(pub usize);
