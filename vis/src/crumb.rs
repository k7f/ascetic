use piet::RenderContext;
use kurbo::{Shape, Line, Rect, RoundedRect, Circle, Arc, BezPath, TranslateScale};
use crate::{Vis, Theme, Style, StyleId, TextLabel};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CrumbId(pub usize);

#[derive(Clone, Copy, Debug)]
pub struct CrumbItem(pub CrumbId, pub TranslateScale, pub Option<StyleId>);

#[derive(Clone, Debug)]
pub enum Crumb {
    Line(Line),
    Rect(Rect),
    RoundedRect(RoundedRect),
    Circle(Circle),
    Arc(Arc),
    Path(BezPath),
    Pin(Circle),
    Label(TextLabel),
}

impl<R: RenderContext> Vis<R> for Line {
    #[inline]
    fn bbox(&self, _rc: &mut R, ts: TranslateScale) -> Rect {
        (ts * *self).bounding_box()
    }

    fn vis(&self, rc: &mut R, ts: TranslateScale, style: Option<&Style>, theme: &Theme) {
        if let Some(stroke) =
            style.and_then(|s| s.get_stroke()).or_else(|| theme.get_default_style().get_stroke())
        {
            rc.stroke(ts * *self, stroke.get_brush(), stroke.get_width());
        }
    }
}

impl<R: RenderContext> Vis<R> for Rect {
    #[inline]
    fn bbox(&self, _rc: &mut R, ts: TranslateScale) -> Rect {
        (ts * *self).bounding_box()
    }

    fn vis(&self, rc: &mut R, ts: TranslateScale, style: Option<&Style>, theme: &Theme) {
        let style = style.unwrap_or_else(|| theme.get_default_style());
        let rect = ts * *self;

        if let Some(brush) = style.get_fill_color() {
            rc.fill(rect, brush);
        } else if let Some(name) = style.get_fill_gradient_name() {
            if let Some(gradient) = theme.get_linear_gradient(name) {
                rc.fill(rect, gradient);
            } else if let Some(gradient) = theme.get_radial_gradient(name) {
                rc.fill(rect, gradient);
            }
        }

        if let Some(border) = style.get_stroke() {
            rc.stroke(rect, border.get_brush(), border.get_width());
        }
    }
}

impl<R: RenderContext> Vis<R> for RoundedRect {
    #[inline]
    fn bbox(&self, _rc: &mut R, ts: TranslateScale) -> Rect {
        (ts * *self).bounding_box()
    }

    fn vis(&self, rc: &mut R, ts: TranslateScale, style: Option<&Style>, theme: &Theme) {
        let style = style.unwrap_or_else(|| theme.get_default_style());
        let rr = ts * *self;

        if let Some(brush) = style.get_fill_color() {
            rc.fill(rr, brush);
        } else if let Some(name) = style.get_fill_gradient_name() {
            if let Some(gradient) = theme.get_linear_gradient(name) {
                rc.fill(rr, gradient);
            } else if let Some(gradient) = theme.get_radial_gradient(name) {
                rc.fill(rr, gradient);
            }
        }

        if let Some(border) = style.get_stroke() {
            rc.stroke(rr, border.get_brush(), border.get_width());
        }
    }
}

impl<R: RenderContext> Vis<R> for Circle {
    #[inline]
    fn bbox(&self, _rc: &mut R, ts: TranslateScale) -> Rect {
        (ts * *self).bounding_box()
    }

    fn vis(&self, rc: &mut R, ts: TranslateScale, style: Option<&Style>, theme: &Theme) {
        let style = style.unwrap_or_else(|| theme.get_default_style());
        let circ = ts * *self;

        if let Some(brush) = style.get_fill_color() {
            rc.fill(circ, brush);
        } else if let Some(name) = style.get_fill_gradient_name() {
            if let Some(gradient) = theme.get_linear_gradient(name) {
                rc.fill(circ, gradient);
            } else if let Some(gradient) = theme.get_radial_gradient(name) {
                rc.fill(circ, gradient);
            }
        }

        if let Some(border) = style.get_stroke() {
            rc.stroke(circ, border.get_brush(), border.get_width());
        }
    }
}

impl<R: RenderContext> Vis<R> for Arc {
    #[inline]
    fn bbox(&self, _rc: &mut R, ts: TranslateScale) -> Rect {
        ts * self.bounding_box()
    }

    fn vis(&self, rc: &mut R, ts: TranslateScale, style: Option<&Style>, theme: &Theme) {
        let style = style.unwrap_or_else(|| theme.get_default_style());
        let path = ts * BezPath::from_vec(self.path_elements(0.1).collect());

        if let Some(brush) = style.get_fill_color() {
            rc.fill(&path, brush);
        } else if let Some(name) = style.get_fill_gradient_name() {
            if let Some(gradient) = theme.get_linear_gradient(name) {
                rc.fill(&path, gradient);
            } else if let Some(gradient) = theme.get_radial_gradient(name) {
                rc.fill(&path, gradient);
            }
        }

        if let Some(border) = style.get_stroke() {
            rc.stroke(&path, border.get_brush(), border.get_width());
        }
    }
}

impl<R: RenderContext> Vis<R> for BezPath {
    #[inline]
    fn bbox(&self, _rc: &mut R, ts: TranslateScale) -> Rect {
        ts * self.bounding_box()
    }

    fn vis(&self, rc: &mut R, ts: TranslateScale, style: Option<&Style>, theme: &Theme) {
        let style = style.unwrap_or_else(|| theme.get_default_style());
        let path = ts * self.clone();

        if let Some(brush) = style.get_fill_color() {
            rc.fill(&path, brush);
        } else if let Some(name) = style.get_fill_gradient_name() {
            if let Some(gradient) = theme.get_linear_gradient(name) {
                rc.fill(&path, gradient);
            } else if let Some(gradient) = theme.get_radial_gradient(name) {
                rc.fill(&path, gradient);
            }
        }

        if let Some(border) = style.get_stroke() {
            rc.stroke(&path, border.get_brush(), border.get_width());
        }
    }
}
