use std::f64::consts::PI;
use kurbo::{Point, Line, Rect, RoundedRect, Circle, Arc, BezPath, Shape, TranslateScale, Size};
use usvg::NodeExt;
use crate::{
    Scene, Theme, Style, StyleId, Stroke, Fill, GradientStop, Gradient, Crumb, CrumbItem,
    Crumbling, TextLabel, VisError,
};

pub use usvg::{Tree, FitTo};
pub use tiny_skia::Pixmap;
pub use resvg::render as render_to_pixmap;

pub trait AsUsvgTree {
    fn as_usvg_tree<S, M>(
        &self,
        theme: &Theme,
        out_size: S,
        out_margin: M,
    ) -> Result<usvg::Tree, VisError>
    where
        S: Into<Size>,
        M: Into<Size>;
}

impl AsUsvgTree for Scene {
    fn as_usvg_tree<S, M>(
        &self,
        theme: &Theme,
        out_size: S,
        out_margin: M,
    ) -> Result<usvg::Tree, VisError>
    where
        S: Into<Size>,
        M: Into<Size>,
    {
        let out_size = out_size.into();
        let out_margin = out_margin.into();
        let out_scale = ((out_size.width - 2. * out_margin.width) / self.get_size().width)
            .min((out_size.height - 2. * out_margin.height) / self.get_size().height);
        let root_ts =
            TranslateScale::translate(out_margin.to_vec2()) * TranslateScale::scale(out_scale);

        let svg_size = usvg::Size::new(out_size.width.round(), out_size.height.round()).unwrap();
        let mut rtree = usvg::Tree::create(usvg::Svg {
            size:     svg_size,
            view_box: usvg::ViewBox {
                rect:   svg_size.to_rect(0.0, 0.0),
                aspect: usvg::AspectRatio::default(),
            },
        });
        let mut root_node = rtree.root();

        for (name, spec) in theme.get_named_gradspecs() {
            let node = spec.as_usvg_node_with_name(name);

            rtree.append_to_defs(node);
        }

        theme.append_background_to_usvg_tree(&mut rtree);

        for (_level, CrumbItem(crumb_id, ts, style_id)) in self.all_visible_crumbs(root_ts)? {
            if let Some(crumb) = self.get_crumb(crumb_id) {
                let (node_kind, more_kinds) = crumb.as_usvg_node_with_style(ts, style_id, theme);

                if let Some(kind) = node_kind {
                    root_node.append(usvg::Node::new(kind));
                }

                for kind in more_kinds {
                    root_node.append(usvg::Node::new(kind));
                }
            } else {
                return Err(VisError::crumb_missing_for_id(crumb_id))
            }
        }

        Ok(rtree)
    }
}

pub trait AsPathData {
    fn as_path_data(&self, ts: TranslateScale) -> usvg::PathData;

    fn as_path_data_and_points(&self, ts: TranslateScale) -> (usvg::PathData, Vec<Point>) {
        (self.as_path_data(ts), Vec::new())
    }
}

impl AsPathData for Crumb {
    #[inline]
    fn as_path_data(&self, ts: TranslateScale) -> usvg::PathData {
        match self {
            Crumb::Line(line) => line.as_path_data(ts),
            Crumb::Rect(rect) => rect.as_path_data(ts),
            Crumb::RoundedRect(rr) => rr.as_path_data(ts),
            Crumb::Circle(circ) => circ.as_path_data(ts),
            Crumb::Arc(arc) => arc.as_path_data(ts),
            Crumb::Path(path) => path.as_path_data(ts),
            Crumb::Pin(_) => usvg::PathData::new(),
            Crumb::Label(label) => label.as_path_data(ts),
        }
    }

    #[inline]
    fn as_path_data_and_points(&self, ts: TranslateScale) -> (usvg::PathData, Vec<Point>) {
        match self {
            Crumb::Line(line) => line.as_path_data_and_points(ts),
            Crumb::Rect(rect) => rect.as_path_data_and_points(ts),
            Crumb::RoundedRect(rr) => rr.as_path_data_and_points(ts),
            Crumb::Circle(circ) => circ.as_path_data_and_points(ts),
            Crumb::Arc(arc) => arc.as_path_data_and_points(ts),
            Crumb::Path(path) => path.as_path_data_and_points(ts),
            Crumb::Pin(_) => (usvg::PathData::new(), Vec::new()),
            Crumb::Label(label) => label.as_path_data_and_points(ts),
        }
    }
}

impl AsPathData for Line {
    #[inline]
    fn as_path_data(&self, ts: TranslateScale) -> usvg::PathData {
        self.as_path_data_and_points(ts).0
    }

    fn as_path_data_and_points(&self, ts: TranslateScale) -> (usvg::PathData, Vec<Point>) {
        let p0 = ts * self.p0;
        let p1 = ts * self.p1;

        (
            usvg::PathData(vec![
                usvg::PathSegment::MoveTo { x: p0.x, y: p0.y },
                usvg::PathSegment::LineTo { x: p1.x, y: p1.y },
            ]),
            vec![p0, p1],
        )
    }
}

impl AsPathData for Rect {
    fn as_path_data(&self, ts: TranslateScale) -> usvg::PathData {
        let rect = ts * *self;

        usvg::PathData(vec![
            usvg::PathSegment::MoveTo { x: rect.x0, y: rect.y0 },
            usvg::PathSegment::LineTo { x: rect.x1, y: rect.y0 },
            usvg::PathSegment::LineTo { x: rect.x1, y: rect.y1 },
            usvg::PathSegment::LineTo { x: rect.x0, y: rect.y1 },
            usvg::PathSegment::ClosePath,
        ])
    }
}

impl AsPathData for RoundedRect {
    fn as_path_data(&self, ts: TranslateScale) -> usvg::PathData {
        let rr = ts * *self;

        if let Some(radius) = rr.radii().as_single_radius() {
            let rect = rr.rect();
            let (rx, ry) = (radius, radius);
            let mut path = usvg::PathData::with_capacity(10);

            path.push_move_to(rect.x0 + rx, rect.y0);

            path.push_line_to(rect.x1 - rx, rect.y0);
            path.push_arc_to(rx, ry, 0.0, false, true, rect.x1, rect.y0 + ry);

            path.push_line_to(rect.x1, rect.y1 - ry);
            path.push_arc_to(rx, ry, 0.0, false, true, rect.x1 - rx, rect.y1);

            path.push_line_to(rect.x0 + rx, rect.y1);
            path.push_arc_to(rx, ry, 0.0, false, true, rect.x0, rect.y1 - ry);

            path.push_line_to(rect.x0, rect.y0 + ry);
            path.push_arc_to(rx, ry, 0.0, false, true, rect.x0 + rx, rect.y0);

            path.push_close_path();

            path
        } else {
            self.rect().as_path_data(ts)
        }
    }
}

impl AsPathData for Circle {
    fn as_path_data(&self, ts: TranslateScale) -> usvg::PathData {
        let center = ts * self.center;
        let (cx, cy) = (center.x, center.y);
        let radius = ts.as_tuple().1 * self.radius;
        let (rx, ry) = (radius, radius);
        let mut path = usvg::PathData::with_capacity(6);

        path.push_move_to(cx + rx, cy);
        path.push_arc_to(rx, ry, 0.0, false, true, cx, cy + ry);
        path.push_arc_to(rx, ry, 0.0, false, true, cx - rx, cy);
        path.push_arc_to(rx, ry, 0.0, false, true, cx, cy - ry);
        path.push_arc_to(rx, ry, 0.0, false, true, cx + rx, cy);
        path.push_close_path();

        path
    }
}

impl AsPathData for Arc {
    #[inline]
    fn as_path_data(&self, ts: TranslateScale) -> usvg::PathData {
        let path = BezPath::from_vec(self.path_elements(0.1).collect());

        path.as_path_data(ts)
    }

    #[inline]
    fn as_path_data_and_points(&self, ts: TranslateScale) -> (usvg::PathData, Vec<Point>) {
        let path = BezPath::from_vec(self.path_elements(0.1).collect());

        path.as_path_data_and_points(ts)
    }
}

impl AsPathData for BezPath {
    #[inline]
    fn as_path_data(&self, ts: TranslateScale) -> usvg::PathData {
        self.as_path_data_and_points(ts).0
    }

    fn as_path_data_and_points(&self, ts: TranslateScale) -> (usvg::PathData, Vec<Point>) {
        let mut out_path = usvg::PathData::with_capacity(self.elements().len());
        let mut out_points = Vec::new();

        for path_el in self.iter() {
            use kurbo::PathEl::*;
            match path_el {
                MoveTo(point) => {
                    let point = ts * point;
                    out_path.push_move_to(point.x, point.y);
                    out_points.push(point);
                }
                LineTo(point) => {
                    let point = ts * point;
                    out_path.push_line_to(point.x, point.y);
                    out_points.push(point);
                }
                QuadTo(point1, point2) => {
                    let point1 = ts * point1;
                    let point2 = ts * point2;
                    out_path.push_quad_to(point1.x, point1.y, point2.x, point2.y);
                    out_points.push(point1);
                    out_points.push(point2);
                }
                CurveTo(point1, point2, point3) => {
                    let point1 = ts * point1;
                    let point2 = ts * point2;
                    let point3 = ts * point3;
                    out_path
                        .push_curve_to(point1.x, point1.y, point2.x, point2.y, point3.x, point3.y);
                    out_points.push(point1);
                    out_points.push(point2);
                    out_points.push(point3);
                }
                ClosePath => out_path.push_close_path(),
            }
        }

        (out_path, out_points)
    }
}

impl AsPathData for TextLabel {
    #[inline]
    fn as_path_data(&self, ts: TranslateScale) -> usvg::PathData {
        self.as_path_data_and_points(ts).0
    }

    fn as_path_data_and_points(&self, _ts: TranslateScale) -> (usvg::PathData, Vec<Point>) {
        // FIXME
        (usvg::PathData::new(), Vec::new())
    }
}

pub trait AsUsvgNodeWithStyle: Crumbling + AsPathData {
    fn as_usvg_node_with_style(
        &self,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> (Option<usvg::NodeKind>, Vec<usvg::NodeKind>) {
        let path_data = self.as_path_data(ts);

        if path_data.0.is_empty() {
            (None, Vec::new())
        } else {
            let (fill, stroke) = theme.get_style_as_usvg(style_id);
            let data = std::rc::Rc::new(path_data);

            (
                Some(usvg::NodeKind::Path(usvg::Path { fill, stroke, data, ..Default::default() })),
                Vec::new(),
            )
        }
    }
}

impl AsUsvgNodeWithStyle for Crumb {
    #[inline]
    fn as_usvg_node_with_style(
        &self,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> (Option<usvg::NodeKind>, Vec<usvg::NodeKind>) {
        match self {
            Crumb::Line(line) => line.as_usvg_node_with_style(ts, style_id, theme),
            Crumb::Rect(rect) => rect.as_usvg_node_with_style(ts, style_id, theme),
            Crumb::RoundedRect(rr) => rr.as_usvg_node_with_style(ts, style_id, theme),
            Crumb::Circle(circ) => circ.as_usvg_node_with_style(ts, style_id, theme),
            Crumb::Arc(arc) => arc.as_usvg_node_with_style(ts, style_id, theme),
            Crumb::Path(path) => path.as_usvg_node_with_style(ts, style_id, theme),
            Crumb::Pin(_) => (None, Vec::new()),
            Crumb::Label(label) => label.as_usvg_node_with_style(ts, style_id, theme),
        }
    }
}

impl AsUsvgNodeWithStyle for Line {
    fn as_usvg_node_with_style(
        &self,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> (Option<usvg::NodeKind>, Vec<usvg::NodeKind>) {
        let mut more_kinds = Vec::new();
        let style = theme.get_style(style_id).unwrap_or_else(|| theme.get_default_style());
        let stroke = style
            .get_stroke()
            .or_else(|| theme.get_default_style().get_stroke())
            .map(|s| s.as_usvg());

        let path_data = {
            let markers = style.get_markers();

            if let Some(marker) =
                markers.get_end_name().and_then(|name| theme.get_marker_by_name(name))
            {
                let (path_data, points) = self.as_path_data_and_points(ts);

                // FIXME precompute marker's path data, clone it here.
                let mut marker_data = marker.get_crumb().as_path_data(TranslateScale::default());

                // FIXME precompute unit width.
                let unit_width = stroke.as_ref().map(|s| s.width.value()).unwrap_or(1.0);
                let angle =
                    marker.get_orient().unwrap_or_else(|| self.end_angle(points.as_slice()));
                let a = angle.cos() * unit_width;
                let b = angle.sin() * unit_width;
                let refx = marker.get_refx();
                let refy = marker.get_refy();
                let p1 = points.last().unwrap();

                marker_data.transform(usvg::Transform::new(
                    a,
                    b,
                    -b,
                    a,
                    p1.x - a * refx + b * refy,
                    p1.y - b * refx - a * refy,
                ));

                let (stroke, fill) = if let Some(style) =
                    marker.get_style_name().and_then(|name| theme.get_style_by_name(name))
                {
                    (style.get_stroke().map(|s| s.as_usvg()), style.get_fill().map(|s| s.as_usvg()))
                } else {
                    (None, None)
                };
                let data = std::rc::Rc::new(marker_data);

                more_kinds.push(usvg::NodeKind::Path(usvg::Path {
                    stroke,
                    fill,
                    data,
                    ..Default::default()
                }));

                path_data
            } else {
                self.as_path_data(ts)
            }
        };

        let data = std::rc::Rc::new(path_data);

        (Some(usvg::NodeKind::Path(usvg::Path { stroke, data, ..Default::default() })), more_kinds)
    }
}

impl AsUsvgNodeWithStyle for Rect {}
impl AsUsvgNodeWithStyle for RoundedRect {}
impl AsUsvgNodeWithStyle for Circle {}

impl AsUsvgNodeWithStyle for Arc {
    fn as_usvg_node_with_style(
        &self,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> (Option<usvg::NodeKind>, Vec<usvg::NodeKind>) {
        let path = BezPath::from_vec(self.path_elements(0.1).collect());

        path.as_usvg_node_with_style(ts, style_id, theme)
    }
}

impl AsUsvgNodeWithStyle for BezPath {
    fn as_usvg_node_with_style(
        &self,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> (Option<usvg::NodeKind>, Vec<usvg::NodeKind>) {
        let mut more_kinds = Vec::new();
        let style = theme.get_style(style_id).unwrap_or_else(|| theme.get_default_style());
        let stroke = style
            .get_stroke()
            .or_else(|| theme.get_default_style().get_stroke())
            .map(|s| s.as_usvg());
        let fill = style.get_fill().map(|f| f.as_usvg());

        let path_data = {
            let markers = style.get_markers();

            if let Some(marker) =
                markers.get_end_name().and_then(|name| theme.get_marker_by_name(name))
            {
                let (path_data, points) = self.as_path_data_and_points(ts);

                // FIXME precompute marker's path data, clone it here.
                let mut marker_data = marker.get_crumb().as_path_data(TranslateScale::default());

                // FIXME precompute unit width.
                let unit_width = stroke.as_ref().map(|s| s.width.value()).unwrap_or(1.0);
                let angle =
                    marker.get_orient().unwrap_or_else(|| self.end_angle(points.as_slice()));
                let a = angle.cos() * unit_width;
                let b = angle.sin() * unit_width;
                let refx = marker.get_refx();
                let refy = marker.get_refy();
                let p1 = points.last().unwrap();

                marker_data.transform(usvg::Transform::new(
                    a,
                    b,
                    -b,
                    a,
                    p1.x - a * refx + b * refy,
                    p1.y - b * refx - a * refy,
                ));

                let (stroke, fill) = if let Some(style) =
                    marker.get_style_name().and_then(|name| theme.get_style_by_name(name))
                {
                    (style.get_stroke().map(|s| s.as_usvg()), style.get_fill().map(|s| s.as_usvg()))
                } else {
                    (None, None)
                };
                let data = std::rc::Rc::new(marker_data);

                more_kinds.push(usvg::NodeKind::Path(usvg::Path {
                    stroke,
                    fill,
                    data,
                    ..Default::default()
                }));

                path_data
            } else {
                self.as_path_data(ts)
            }
        };

        let data = std::rc::Rc::new(path_data);

        (
            Some(usvg::NodeKind::Path(usvg::Path { stroke, fill, data, ..Default::default() })),
            more_kinds,
        )
    }
}

impl AsUsvgNodeWithStyle for TextLabel {}

pub trait AsUsvgNodeWithName {
    fn as_usvg_node_with_name<S: AsRef<str>>(&self, name: S) -> usvg::NodeKind;
}

impl AsUsvgNodeWithName for Gradient {
    fn as_usvg_node_with_name<S: AsRef<str>>(&self, name: S) -> usvg::NodeKind {
        match self {
            Gradient::Linear(start, end, stops) => {
                let start = start.resolve(Rect::new(0., 0., 1., 1.));
                let end = end.resolve(Rect::new(0., 0., 1., 1.));

                usvg::NodeKind::LinearGradient(usvg::LinearGradient {
                    id:   name.as_ref().into(),
                    x1:   start.x,
                    y1:   start.y,
                    x2:   end.x,
                    y2:   end.y,
                    base: usvg::BaseGradient {
                        units:         usvg::Units::ObjectBoundingBox,
                        transform:     usvg::Transform::default(),
                        spread_method: usvg::SpreadMethod::Pad,
                        stops:         stops.iter().map(|stop| stop.as_usvg()).collect(),
                    },
                })
            }
            Gradient::Radial(radius, stops) => {
                usvg::NodeKind::RadialGradient(usvg::RadialGradient {
                    id:   name.as_ref().into(),
                    r:    (*radius).into(),
                    cx:   0.5,
                    cy:   0.5,
                    fx:   0.5,
                    fy:   0.5,
                    base: usvg::BaseGradient {
                        units:         usvg::Units::ObjectBoundingBox,
                        transform:     usvg::Transform::default(),
                        spread_method: usvg::SpreadMethod::Pad,
                        stops:         stops.iter().map(|stop| stop.as_usvg()).collect(),
                    },
                })
            }
        }
    }
}

pub trait AsUsvgTheme {
    fn get_style_as_usvg(
        &self,
        style_id: Option<StyleId>,
    ) -> (Option<usvg::Fill>, Option<usvg::Stroke>);

    fn get_stroke_as_usvg(&self, style_id: Option<StyleId>) -> Option<usvg::Stroke>;

    fn append_background_to_usvg_tree(&self, rtree: &mut usvg::Tree);
}

impl AsUsvgTheme for Theme {
    #[inline]
    fn get_style_as_usvg(
        &self,
        style_id: Option<StyleId>,
    ) -> (Option<usvg::Fill>, Option<usvg::Stroke>) {
        self.get_style(style_id).map(|style| style.as_usvg()).unwrap_or((None, None))
    }

    #[inline]
    fn get_stroke_as_usvg(&self, style_id: Option<StyleId>) -> Option<usvg::Stroke> {
        self.get_stroke(style_id)
            .or_else(|| self.get_default_style().get_stroke())
            .map(|s| s.as_usvg())
    }

    #[inline]
    fn append_background_to_usvg_tree(&self, rtree: &mut usvg::Tree) {
        let size = rtree.svg_node().size;
        let fill = Fill::Color(self.get_bg_color()).as_usvg();
        let path_data = usvg::PathData::from_rect(size.to_rect(0.0, 0.0));
        let path = usvg::Path {
            fill: Some(fill),
            data: std::rc::Rc::new(path_data),
            ..Default::default()
        };
        rtree.root().append_kind(usvg::NodeKind::Path(path));
    }
}

pub trait AsUsvgStyle {
    fn as_usvg(&self) -> (Option<usvg::Fill>, Option<usvg::Stroke>);
}

impl AsUsvgStyle for Style {
    #[inline]
    fn as_usvg(&self) -> (Option<usvg::Fill>, Option<usvg::Stroke>) {
        (self.get_fill().map(|f| f.as_usvg()), self.get_stroke().map(|s| s.as_usvg()))
    }
}

pub trait AsUsvgStroke {
    fn as_usvg(&self) -> usvg::Stroke;
}

impl AsUsvgStroke for Stroke {
    fn as_usvg(&self) -> usvg::Stroke {
        let (red, green, blue, alpha) = self.get_brush().as_rgba8();

        if alpha == 0xff {
            usvg::Stroke {
                paint: usvg::Paint::Color(usvg::Color::new_rgb(red, green, blue)),
                width: self.get_width().into(),
                ..Default::default()
            }
        } else {
            usvg::Stroke {
                paint: usvg::Paint::Color(usvg::Color::new_rgb(red, green, blue)),
                opacity: (alpha as f64 / 255.0).into(),
                width: self.get_width().into(),
                ..Default::default()
            }
        }
    }
}

pub trait AsUsvgFill {
    fn as_usvg(&self) -> usvg::Fill;
}

impl AsUsvgFill for Fill {
    fn as_usvg(&self) -> usvg::Fill {
        let (paint, alpha) = match self {
            Fill::Color(color) => {
                let (red, green, blue, alpha) = color.as_rgba8();

                (usvg::Paint::Color(usvg::Color::new_rgb(red, green, blue)), alpha)
            }
            Fill::Linear(name) | Fill::Radial(name) => (usvg::Paint::Link(name.into()), 0xff),
        };

        if alpha == 0xff {
            usvg::Fill { paint, ..Default::default() }
        } else {
            usvg::Fill { paint, opacity: (alpha as f64 / 255.0).into(), ..Default::default() }
        }
    }
}

pub trait AsUsvgStop {
    fn as_usvg(&self) -> usvg::Stop;
}

impl AsUsvgStop for GradientStop {
    fn as_usvg(&self) -> usvg::Stop {
        let (red, green, blue, alpha) = self.color.as_rgba8();

        usvg::Stop {
            offset:  usvg::StopOffset::new(self.pos.into()),
            color:   usvg::Color::new_rgb(red, green, blue),
            opacity: if alpha == 0xff { 1.0.into() } else { (alpha as f64 / 255.0).into() },
        }
    }
}
