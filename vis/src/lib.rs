mod crumb;
mod group;
mod style;
mod font;
mod theme;
mod scene;
mod tweener;
mod joint;
mod text;
pub mod backend;

pub use crumb::{Crumb, CrumbId, CrumbItem};
pub use group::{Group, GroupId, GroupItem};
pub use style::{Style, StyleId, Stroke, Fill, GradSpec, Marker, MarkerId};
pub use font::Font;
pub use scene::Scene;
pub use theme::{Theme, Variation};
pub use tweener::{Tweener, Tweenable, Steppable, LinearEasing};
pub use joint::Joint;
pub use text::TextLabel;

pub use piet::{Color, UnitPoint};
pub use kurbo::{self, Line, Rect, RoundedRect, Circle, TranslateScale, Vec2};

pub trait Vis {
    fn bbox(&self, ts: TranslateScale, style: Option<&Style>, theme: &Theme) -> Rect;

    fn vis<R: piet::RenderContext>(
        &self,
        rc: &mut R,
        ts: TranslateScale,
        style: Option<&Style>,
        theme: &Theme,
    );
}
