use piet::{RenderContext, Text, TextLayoutBuilder, FontFamily};
use kurbo::{Shape, Point, Line, Rect, RoundedRect, Circle, Arc, BezPath, TranslateScale};
use crate::{Vis, Theme, Style, StyleId, Font, TextLabel};

#[derive(Clone, Copy, Debug)]
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

impl Vis for Crumb {
    fn bbox(&self, ts: TranslateScale, style: Option<&Style>, theme: &Theme) -> Rect {
        match self {
            Crumb::Line(line) => line.bbox(ts, style, theme),
            Crumb::Rect(rect) => rect.bbox(ts, style, theme),
            Crumb::RoundedRect(rr) => rr.bbox(ts, style, theme),
            Crumb::Circle(circ) => circ.bbox(ts, style, theme),
            Crumb::Arc(arc) => arc.bbox(ts, style, theme),
            Crumb::Path(path) => path.bbox(ts, style, theme),
            Crumb::Pin(pin) => pin.bbox(ts, style, theme),
            Crumb::Label(label) => label.bbox(ts, style, theme),
        }
    }

    fn vis<R: RenderContext>(
        &self,
        rc: &mut R,
        ts: TranslateScale,
        style: Option<&Style>,
        theme: &Theme,
    ) {
        match self {
            Crumb::Line(line) => line.vis(rc, ts, style, theme),
            Crumb::Rect(rect) => rect.vis(rc, ts, style, theme),
            Crumb::RoundedRect(rr) => rr.vis(rc, ts, style, theme),
            Crumb::Circle(circ) => circ.vis(rc, ts, style, theme),
            Crumb::Arc(arc) => arc.vis(rc, ts, style, theme),
            Crumb::Path(path) => path.vis(rc, ts, style, theme),
            Crumb::Pin(_) => {}
            Crumb::Label(label) => label.vis(rc, ts, style, theme),
        }
    }
}

impl Vis for Line {
    #[inline]
    fn bbox(&self, ts: TranslateScale, _style: Option<&Style>, _theme: &Theme) -> Rect {
        (ts * *self).bounding_box()
    }

    fn vis<R: RenderContext>(
        &self,
        rc: &mut R,
        ts: TranslateScale,
        style: Option<&Style>,
        theme: &Theme,
    ) {
        if let Some(stroke) =
            style.and_then(|s| s.get_stroke()).or_else(|| theme.get_default_style().get_stroke())
        {
            rc.stroke(ts * *self, stroke.get_brush(), stroke.get_width());
        }
    }
}

impl Vis for Rect {
    #[inline]
    fn bbox(&self, ts: TranslateScale, _style: Option<&Style>, _theme: &Theme) -> Rect {
        (ts * *self).bounding_box()
    }

    fn vis<R: RenderContext>(
        &self,
        rc: &mut R,
        ts: TranslateScale,
        style: Option<&Style>,
        theme: &Theme,
    ) {
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

impl Vis for RoundedRect {
    #[inline]
    fn bbox(&self, ts: TranslateScale, _style: Option<&Style>, _theme: &Theme) -> Rect {
        (ts * *self).bounding_box()
    }

    fn vis<R: RenderContext>(
        &self,
        rc: &mut R,
        ts: TranslateScale,
        style: Option<&Style>,
        theme: &Theme,
    ) {
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

impl Vis for Circle {
    #[inline]
    fn bbox(&self, ts: TranslateScale, _style: Option<&Style>, _theme: &Theme) -> Rect {
        (ts * *self).bounding_box()
    }

    fn vis<R: RenderContext>(
        &self,
        rc: &mut R,
        ts: TranslateScale,
        style: Option<&Style>,
        theme: &Theme,
    ) {
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

impl Vis for Arc {
    #[inline]
    fn bbox(&self, ts: TranslateScale, _style: Option<&Style>, _theme: &Theme) -> Rect {
        ts * self.bounding_box()
    }

    fn vis<R: RenderContext>(
        &self,
        rc: &mut R,
        ts: TranslateScale,
        style: Option<&Style>,
        theme: &Theme,
    ) {
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

impl Vis for BezPath {
    #[inline]
    fn bbox(&self, ts: TranslateScale, _style: Option<&Style>, _theme: &Theme) -> Rect {
        ts * self.bounding_box()
    }

    fn vis<R: RenderContext>(
        &self,
        rc: &mut R,
        ts: TranslateScale,
        style: Option<&Style>,
        theme: &Theme,
    ) {
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

impl Vis for TextLabel {
    #[inline]
    fn bbox(&self, ts: TranslateScale, _style: Option<&Style>, _theme: &Theme) -> Rect {
        // FIXME
        ts * self.bounding_box(Font::new_serif())
    }

    fn vis<R: RenderContext>(
        &self,
        rc: &mut R,
        ts: TranslateScale,
        _style: Option<&Style>,
        _theme: &Theme,
    ) {
        let origin: Point = (self.x, self.y).into();
        let text = rc.text();
        let font = text.font_family("Arial").unwrap_or(FontFamily::SANS_SERIF);

        if let Ok(mut layout) = text.new_text_layout(self.body.clone()).font(font, 12.0).build() {
            rc.draw_text(&mut layout, ts * origin);
        }
    }
}
