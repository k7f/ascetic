use std::io;
use piet_common::{
    RenderContext,
    kurbo::{Shape, Line, Rect, RoundedRect, Circle, TranslateScale},
};
use crate::{Vis, Theme, StyleId, WriteSvg};

#[derive(Clone, Debug)]
pub enum Prim {
    Line(Line),
    Rect(Rect),
    RoundedRect(RoundedRect),
    Circle(Circle),
}

impl Vis for Prim {
    fn bbox(&self, ts: TranslateScale) -> Rect {
        match *self {
            Prim::Line(line) => line.bbox(ts),
            Prim::Rect(rect) => rect.bbox(ts),
            Prim::RoundedRect(rr) => rr.bbox(ts),
            Prim::Circle(circ) => circ.bbox(ts),
        }
    }

    /// In order to implement `Shape` for `Prim`, associated type
    /// `BezPathIter` would have to be generic...
    fn vis<R: RenderContext>(
        &self,
        rc: &mut R,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) {
        match *self {
            Prim::Line(line) => line.vis(rc, ts, style_id, theme),
            Prim::Rect(rect) => rect.vis(rc, ts, style_id, theme),
            Prim::RoundedRect(rr) => rr.vis(rc, ts, style_id, theme),
            Prim::Circle(circ) => circ.vis(rc, ts, style_id, theme),
        }
    }

    fn write_svg_with_style<W: io::Write>(
        &self,
        mut svg: W,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> io::Result<()> {
        match self {
            Prim::Line(line) => line.write_svg_with_style(&mut svg, ts, style_id, theme),
            Prim::Rect(rect) => rect.write_svg_with_style(&mut svg, ts, style_id, theme),
            Prim::RoundedRect(rr) => rr.write_svg_with_style(&mut svg, ts, style_id, theme),
            Prim::Circle(circ) => circ.write_svg_with_style(&mut svg, ts, style_id, theme),
        }
    }
}

impl Vis for Line {
    #[inline]
    fn bbox(&self, ts: TranslateScale) -> Rect {
        (ts * *self).bounding_box()
    }

    fn vis<R: RenderContext>(
        &self,
        rc: &mut R,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) {
        if let Some(stroke) = theme.get_stroke(style_id).or_else(|| theme.get_default_stroke()) {
            rc.stroke(ts * *self, stroke.get_brush(), stroke.get_width());
        }
    }

    fn write_svg_with_style<W: io::Write>(
        &self,
        mut svg: W,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> io::Result<()> {
        let p0 = ts * self.p0;
        let p1 = ts * self.p1;

        write!(svg, "  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" ", p0.x, p0.y, p1.x, p1.y)?;

        if let Some(stroke) = theme.get_stroke(style_id).or_else(|| theme.get_default_stroke()) {
            stroke.write_svg(&mut svg)?;
        }

        writeln!(svg, "/>")
    }
}

impl Vis for Rect {
    #[inline]
    fn bbox(&self, ts: TranslateScale) -> Rect {
        (ts * *self).bounding_box()
    }

    fn vis<R: RenderContext>(
        &self,
        rc: &mut R,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) {
        if let Some(style) = theme.get_style(style_id).or_else(|| theme.get_default_style()) {
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

    fn write_svg_with_style<W: io::Write>(
        &self,
        mut svg: W,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> io::Result<()> {
        let rect = ts * *self;

        write!(
            svg,
            "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" ",
            rect.x0,
            rect.y0,
            rect.width(),
            rect.height()
        )?;

        if let Some(style) = theme.get_style(style_id) {
            style.write_svg(&mut svg)?;
        }

        writeln!(svg, "/>")
    }
}

impl Vis for RoundedRect {
    #[inline]
    fn bbox(&self, ts: TranslateScale) -> Rect {
        (ts * *self).bounding_box()
    }

    fn vis<R: RenderContext>(
        &self,
        rc: &mut R,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) {
        if let Some(style) = theme.get_style(style_id).or_else(|| theme.get_default_style()) {
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

    fn write_svg_with_style<W: io::Write>(
        &self,
        mut svg: W,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> io::Result<()> {
        let rr = ts * *self;
        let rect = &rr.rect();

        write!(
            svg,
            "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" ",
            rect.x0,
            rect.y0,
            rect.width(),
            rect.height(),
            rr.radius()
        )?;

        if let Some(style) = theme.get_style(style_id) {
            style.write_svg(&mut svg)?;
        }

        writeln!(svg, "/>")
    }
}

impl Vis for Circle {
    #[inline]
    fn bbox(&self, ts: TranslateScale) -> Rect {
        (ts * *self).bounding_box()
    }

    fn vis<R: RenderContext>(
        &self,
        rc: &mut R,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) {
        if let Some(style) = theme.get_style(style_id).or_else(|| theme.get_default_style()) {
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

    fn write_svg_with_style<W: io::Write>(
        &self,
        mut svg: W,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> io::Result<()> {
        let center = ts * self.center;
        let radius = ts.as_tuple().1 * self.radius;

        write!(svg, "  <circle cx=\"{}\" cy=\"{}\" r=\"{}\" ", center.x, center.y, radius)?;

        if let Some(style) = theme.get_style(style_id) {
            style.write_svg(&mut svg)?;
        }

        writeln!(svg, "/>")
    }
}

// let path_iter = self.to_bez_path(1e-3);
// write!(svg, "  <path d=\"")?;
// path.write_to(&mut svg)?;
// write!(svg, "\" ")?;
