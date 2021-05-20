use piet::RenderContext;
use kurbo::{Shape, Line, Rect, RoundedRect, Circle, TranslateScale};
use crate::{Vis, AsUsvgNodeWithStyle, Theme, StyleId};

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

impl AsUsvgNodeWithStyle for Crumb {
    fn as_usvg_node_with_style(
        &self,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> usvg::NodeKind {
        match self {
            Crumb::Line(line) => line.as_usvg_node_with_style(ts, style_id, theme),
            Crumb::Rect(rect) => rect.as_usvg_node_with_style(ts, style_id, theme),
            Crumb::RoundedRect(rr) => rr.as_usvg_node_with_style(ts, style_id, theme),
            Crumb::Circle(circ) => circ.as_usvg_node_with_style(ts, style_id, theme),
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

impl AsUsvgNodeWithStyle for Line {
    fn as_usvg_node_with_style(
        &self,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> usvg::NodeKind {
        let p0 = ts * self.p0;
        let p1 = ts * self.p1;
        let stroke = theme.get_stroke_as_usvg(style_id);
        let segments = vec![
            usvg::PathSegment::MoveTo { x: p0.x, y: p0.y },
            usvg::PathSegment::LineTo { x: p1.x, y: p1.y },
        ];
        let data = std::rc::Rc::new(usvg::PathData(segments));

        usvg::NodeKind::Path(usvg::Path { stroke, data, ..Default::default() })
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

impl AsUsvgNodeWithStyle for Rect {
    fn as_usvg_node_with_style(
        &self,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> usvg::NodeKind {
        let rect = ts * *self;
        let (fill, stroke) = theme.get_style_as_usvg(style_id);
        let segments = vec![
            usvg::PathSegment::MoveTo { x: rect.x0, y: rect.y0 },
            usvg::PathSegment::LineTo { x: rect.x1, y: rect.y0 },
            usvg::PathSegment::LineTo { x: rect.x1, y: rect.y1 },
            usvg::PathSegment::LineTo { x: rect.x0, y: rect.y1 },
            usvg::PathSegment::ClosePath,
        ];
        let data = std::rc::Rc::new(usvg::PathData(segments));

        usvg::NodeKind::Path(usvg::Path { fill, stroke, data, ..Default::default() })
    }
}

fn rr_data(x: f64, y: f64, width: f64, height: f64, rx: f64, ry: f64) -> usvg::PathData {
    let mut path = usvg::PathData::with_capacity(10);
    path.push_move_to(x + rx, y);

    path.push_line_to(x + width - rx, y);
    path.push_arc_to(rx, ry, 0.0, false, true, x + width, y + ry);

    path.push_line_to(x + width, y + height - ry);
    path.push_arc_to(rx, ry, 0.0, false, true, x + width - rx, y + height);

    path.push_line_to(x + rx, y + height);
    path.push_arc_to(rx, ry, 0.0, false, true, x, y + height - ry);

    path.push_line_to(x, y + ry);
    path.push_arc_to(rx, ry, 0.0, false, true, x + rx, y);

    path.push_close_path();

    path
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

impl AsUsvgNodeWithStyle for RoundedRect {
    fn as_usvg_node_with_style(
        &self,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> usvg::NodeKind {
        let rr = ts * *self;
        let rect = &rr.rect();
        let radius = rr.radius();
        let (fill, stroke) = theme.get_style_as_usvg(style_id);
        let data = std::rc::Rc::new(rr_data(
            rect.x0,
            rect.y0,
            rect.width(),
            rect.height(),
            radius,
            radius,
        ));

        usvg::NodeKind::Path(usvg::Path { fill, stroke, data, ..Default::default() })
    }
}

fn ellipse_data(cx: f64, cy: f64, rx: f64, ry: f64) -> usvg::PathData {
    let mut path = usvg::PathData::with_capacity(6);

    path.push_move_to(cx + rx, cy);
    path.push_arc_to(rx, ry, 0.0, false, true, cx, cy + ry);
    path.push_arc_to(rx, ry, 0.0, false, true, cx - rx, cy);
    path.push_arc_to(rx, ry, 0.0, false, true, cx, cy - ry);
    path.push_arc_to(rx, ry, 0.0, false, true, cx + rx, cy);
    path.push_close_path();

    path
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

impl AsUsvgNodeWithStyle for Circle {
    fn as_usvg_node_with_style(
        &self,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> usvg::NodeKind {
        let center = ts * self.center;
        let radius = ts.as_tuple().1 * self.radius;
        let (fill, stroke) = theme.get_style_as_usvg(style_id);
        let data = std::rc::Rc::new(ellipse_data(center.x, center.y, radius, radius));

        usvg::NodeKind::Path(usvg::Path { fill, stroke, data, ..Default::default() })
    }
}
