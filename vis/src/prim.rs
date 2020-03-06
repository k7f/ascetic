use std::io;
use piet_common::{
    RenderContext,
    kurbo::{Line, Rect, RoundedRect, TranslateScale},
};
use crate::{Vis, Theme, StyleId, WriteSvg, WriteSvgWithStyle};

#[derive(Clone, Debug)]
pub enum Prim {
    Line(Line),
    Rect(Rect),
    RoundedRect(RoundedRect),
}

impl Prim {
    /// In order to implement `Shape` for `Prim`, associated type
    /// `BezPathIter` would have to be generic...
    pub fn vis<R: RenderContext>(
        &self,
        rc: &mut R,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) {
        match *self {
            Prim::Line(line) => line.vis(rc, ts, style_id, theme),
            Prim::Rect(rect) => rect.vis(rc, ts, style_id, theme),
            Prim::RoundedRect(rect) => rect.vis(rc, ts, style_id, theme),
        }
    }
}

impl WriteSvgWithStyle for Prim {
    fn write_svg_with_style<W: io::Write>(
        &self,
        mut svg: W,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> io::Result<()> {
        match *self {
            Prim::Line(line) => line.write_svg_with_style(&mut svg, ts, style_id, theme),
            Prim::Rect(rect) => rect.write_svg_with_style(&mut svg, ts, style_id, theme),
            Prim::RoundedRect(rect) => rect.write_svg_with_style(&mut svg, ts, style_id, theme),
        }
    }
}

impl<R: RenderContext> Vis<R> for Line {
    fn vis(&self, rc: &mut R, ts: TranslateScale, style_id: Option<StyleId>, theme: &Theme) {
        if let Some(stroke) = theme.get_stroke(style_id).or_else(|| theme.get_default_stroke()) {
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

        if let Some(stroke) = theme.get_stroke(style_id).or_else(|| theme.get_default_stroke()) {
            stroke.write_svg(&mut svg)?;
        }

        writeln!(svg, "/>")
    }
}

impl<R: RenderContext> Vis<R> for Rect {
    fn vis(&self, rc: &mut R, ts: TranslateScale, style_id: Option<StyleId>, theme: &Theme) {
        if let Some(style) = theme.get_style(style_id).or_else(|| theme.get_default_style()) {
            let rect = ts * *self;

            if let Some(brush) = style.get_fill_color() {
                rc.fill(rect, brush);
            } else if let Some(name) = style.get_fill_gradient_name() {
                if let Some(gradient) = theme.get_gradient(name) {
                    rc.fill(rect, gradient);
                }
            }

            if let Some(border) = style.get_stroke() {
                rc.stroke(rect, border.get_brush(), border.get_width());
            }
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

impl<R: RenderContext> Vis<R> for RoundedRect {
    fn vis(&self, rc: &mut R, ts: TranslateScale, style_id: Option<StyleId>, theme: &Theme) {
        if let Some(style) = theme.get_style(style_id).or_else(|| theme.get_default_style()) {
            let rect = ts * *self;

            if let Some(brush) = style.get_fill_color() {
                rc.fill(rect, brush);
            } else if let Some(name) = style.get_fill_gradient_name() {
                if let Some(gradient) = theme.get_gradient(name) {
                    rc.fill(rect, gradient);
                }
            }

            if let Some(border) = style.get_stroke() {
                rc.stroke(rect, border.get_brush(), border.get_width());
            }
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

// let path_iter = self.to_bez_path(1e-3);
// write!(svg, "  <path d=\"")?;
// path.write_to(&mut svg)?;
// write!(svg, "\" ")?;
