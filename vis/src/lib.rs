mod crumb;
mod group;
mod style;
mod theme;
mod scene;
mod tweener;
pub mod backend;

pub use crumb::{Crumb, CrumbId, CrumbItem};
pub use group::{Group, GroupId, GroupItem};
pub use style::{Style, StyleId, Stroke, Fill, GradSpec};
pub use scene::Scene;
pub use theme::{Theme, Variation};
pub use tweener::{Tweener, Tweenable, Steppable, LinearEasing};

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

pub trait AsUsvgNodeWithStyle {
    fn as_usvg_node_with_style(
        &self,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> usvg::NodeKind;
}

pub trait AsUsvgNodeWithName {
    fn as_usvg_node_with_name<S: AsRef<str>>(&self, name: S) -> usvg::NodeKind;
}

pub trait AsUsvgStyle {
    fn as_usvg(&self) -> (Option<usvg::Fill>, Option<usvg::Stroke>);
}

pub trait AsUsvgStroke {
    fn as_usvg(&self) -> usvg::Stroke;
}

pub trait AsUsvgFill {
    fn as_usvg(&self) -> usvg::Fill;
}

pub trait AsUsvgStop {
    fn as_usvg(&self) -> usvg::Stop;
}
