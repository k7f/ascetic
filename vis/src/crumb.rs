use std::io;
use piet::RenderContext;
use kurbo::{Shape, Line, Rect, RoundedRect, Circle, TranslateScale};
use crate::{Vis, WriteSvgWithStyle, Theme, StyleId, WriteSvg};

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
}

impl Vis for Crumb {
    fn bbox(&self, ts: TranslateScale) -> Rect {
        match *self {
            Crumb::Line(line) => line.bbox(ts),
            Crumb::Rect(rect) => rect.bbox(ts),
            Crumb::RoundedRect(rr) => rr.bbox(ts),
            Crumb::Circle(circ) => circ.bbox(ts),
        }
    }

    fn vis<R: RenderContext>(
        &self,
        rc: &mut R,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) {
        match *self {
            Crumb::Line(line) => line.vis(rc, ts, style_id, theme),
            Crumb::Rect(rect) => rect.vis(rc, ts, style_id, theme),
            Crumb::RoundedRect(rr) => rr.vis(rc, ts, style_id, theme),
            Crumb::Circle(circ) => circ.vis(rc, ts, style_id, theme),
        }
    }
}

impl WriteSvgWithStyle for Crumb {
    fn write_svg_with_style<W: io::Write>(
        &self,
        mut svg: W,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> io::Result<()> {
        match self {
            Crumb::Line(line) => line.write_svg_with_style(&mut svg, ts, style_id, theme),
            Crumb::Rect(rect) => rect.write_svg_with_style(&mut svg, ts, style_id, theme),
            Crumb::RoundedRect(rr) => rr.write_svg_with_style(&mut svg, ts, style_id, theme),
            Crumb::Circle(circ) => circ.write_svg_with_style(&mut svg, ts, style_id, theme),
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
        if let Some(stroke) =
            theme.get_stroke(style_id).or_else(|| theme.get_default_style().get_stroke())
        {
            rc.stroke(ts * *self, stroke.get_brush(), stroke.get_width());
        }
    }
}

impl WriteSvgWithStyle for Line {
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

        if let Some(stroke) =
            theme.get_stroke(style_id).or_else(|| theme.get_default_style().get_stroke())
        {
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
        let style = theme.get_style(style_id).unwrap_or_else(|| theme.get_default_style());
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

impl WriteSvgWithStyle for Rect {
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
        let style = theme.get_style(style_id).unwrap_or_else(|| theme.get_default_style());
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

impl WriteSvgWithStyle for RoundedRect {
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
        let style = theme.get_style(style_id).unwrap_or_else(|| theme.get_default_style());
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

impl WriteSvgWithStyle for Circle {
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
