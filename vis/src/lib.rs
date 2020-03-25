#![feature(iter_map_while)]

mod crumb;
mod group;
mod style;
mod theme;
mod scene;
mod tweener;
mod backend;

pub use crumb::{Crumb, CrumbId, CrumbItem};
pub use group::{Group, GroupId, GroupItem};
pub use style::{Style, StyleId, Stroke, Fill, GradSpec};
pub use scene::Scene;
pub use theme::{Theme, Variation};
pub use tweener::{Tweener, Tweenable, Steppable, LinearEasing};
pub use backend::cairo::BitmapDevice as CairoBitmapDevice;

use std::io;
use piet::RenderContext;

pub use piet::{self, Color, UnitPoint, ImageFormat};
pub use kurbo::{self, Line, Rect, RoundedRect, Circle, TranslateScale, Vec2};

pub trait Vis {
    fn bbox(&self, ts: TranslateScale) -> Rect;

    fn vis<R: RenderContext>(
        &self,
        rc: &mut R,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    );
}

pub trait WriteSvgWithStyle {
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
