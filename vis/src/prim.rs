use piet_common::{
    RenderContext,
    kurbo::{Line, Rect, RoundedRect},
};
use crate::{Vis, Theme, StyleId};

#[derive(Clone, Debug)]
pub enum Prim {
    Line(Line),
    Rect(Rect),
    RoundedRect(RoundedRect),
}

impl<R: RenderContext> Vis<R> for Line {
    fn vis(&self, style_id: Option<StyleId>, theme: &Theme, rc: &mut R) {
        if let Some(stroke) = theme.get_stroke(style_id).or_else(|| theme.get_default_stroke()) {
            rc.stroke(self, stroke.get_brush(), stroke.get_width());
        }
    }
}

impl<R: RenderContext> Vis<R> for Rect {
    fn vis(&self, style_id: Option<StyleId>, theme: &Theme, rc: &mut R) {
        if let Some(style) = theme.get_style(style_id).or_else(|| theme.get_default_style()) {
            if let Some(brush) = style.get_fill_color() {
                rc.fill(self, brush);
            } else if let Some(name) = style.get_fill_gradient_name() {
                if let Some(gradient) = theme.get_gradient(name) {
                    rc.fill(self, gradient);
                }
            }

            if let Some(border) = style.get_stroke() {
                rc.stroke(self, border.get_brush(), border.get_width());
            }
        }
    }
}

impl<R: RenderContext> Vis<R> for RoundedRect {
    fn vis(&self, style_id: Option<StyleId>, theme: &Theme, rc: &mut R) {
        if let Some(style) = theme.get_style(style_id).or_else(|| theme.get_default_style()) {
            if let Some(brush) = style.get_fill_color() {
                rc.fill(self, brush);
            } else if let Some(name) = style.get_fill_gradient_name() {
                if let Some(gradient) = theme.get_gradient(name) {
                    rc.fill(self, gradient);
                }
            }

            if let Some(border) = style.get_stroke() {
                rc.stroke(self, border.get_brush(), border.get_width());
            }
        }
    }
}
