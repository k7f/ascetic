use std::{borrow::Cow, ops::RangeBounds, f64::consts::PI};
use kurbo::{Point, Line, Rect, RoundedRect, Circle, Arc, BezPath, Shape, TranslateScale, Size, Affine};
use piet::{
    Color, GradientStop, Error, FixedGradient, FontFamily, HitTestPoint, HitTestPosition, Image,
    ImageFormat, InterpolationMode, IntoBrush, LineMetric, RenderContext, StrokeStyle, Text,
    TextAlignment, TextAttribute, TextLayout, TextLayoutBuilder, TextStorage,
};
use usvg::NodeExt;
use crate::{Scene, Theme, Style, StyleId, Stroke, Fill, GradSpec, Crumb, CrumbItem};

pub use usvg::{Tree, FitTo};
pub use tiny_skia::Pixmap;
pub use resvg::render as render_to_pixmap;

pub trait AsUsvgTree {
    fn as_usvg_tree<S, M>(&self, theme: &Theme, out_size: S, out_margin: M) -> usvg::Tree
    where
        S: Into<Size>,
        M: Into<Size>;
}

impl AsUsvgTree for Scene {
    fn as_usvg_tree<S, M>(&self, theme: &Theme, out_size: S, out_margin: M) -> usvg::Tree
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

        for CrumbItem(crumb_id, ts, style_id) in self.all_crumbs(root_ts) {
            if let Some(crumb) = self.get_crumb(crumb_id) {
                let (node_kind, more_kinds) = crumb.as_usvg_node_with_style(ts, style_id, theme);
                let node = usvg::Node::new(node_kind);

                root_node.append(node);

                for kind in more_kinds {
                    let node = usvg::Node::new(kind);
                    root_node.append(node);
                }
            } else {
                // FIXME
                panic!()
            }
        }

        rtree
    }
}

pub trait AsUsvgNodeWithStyle {
    fn end_angle(&self, points: &[Point]) -> f64 {
        if let Some((p1, head)) = points.split_last() {
            if let Some(p0) = head.last() {
                let angle = (p1.y - p0.y).atan2(p1.x - p0.x);

                if !angle.is_nan() {
                    if angle < 0.0 {
                        return angle % (PI * 2.0) + PI * 2.0
                    } else {
                        return angle % (PI * 2.0)
                    }
                }
            }
        }
        0.0
    }

    fn as_path_data(&self, ts: TranslateScale) -> usvg::PathData;

    fn as_path_data_and_points(&self, ts: TranslateScale) -> (usvg::PathData, Vec<Point>) {
        (self.as_path_data(ts), Vec::new())
    }

    fn as_usvg_node_with_style(
        &self,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> (usvg::NodeKind, Vec<usvg::NodeKind>) {
        let (fill, stroke) = theme.get_style_as_usvg(style_id);
        let data = std::rc::Rc::new(self.as_path_data(ts));

        (usvg::NodeKind::Path(usvg::Path { fill, stroke, data, ..Default::default() }), Vec::new())
    }
}

impl AsUsvgNodeWithStyle for Crumb {
    fn end_angle(&self, points: &[Point]) -> f64 {
        match self {
            Crumb::Line(line) => line.end_angle(points),
            Crumb::Rect(rect) => rect.end_angle(points),
            Crumb::RoundedRect(rr) => rr.end_angle(points),
            Crumb::Circle(circ) => circ.end_angle(points),
            Crumb::Arc(arc) => arc.end_angle(points),
            Crumb::Path(path) => path.end_angle(points),
        }
    }

    fn as_path_data(&self, ts: TranslateScale) -> usvg::PathData {
        match self {
            Crumb::Line(line) => line.as_path_data(ts),
            Crumb::Rect(rect) => rect.as_path_data(ts),
            Crumb::RoundedRect(rr) => rr.as_path_data(ts),
            Crumb::Circle(circ) => circ.as_path_data(ts),
            Crumb::Arc(arc) => arc.as_path_data(ts),
            Crumb::Path(path) => path.as_path_data(ts),
        }
    }

    fn as_path_data_and_points(&self, ts: TranslateScale) -> (usvg::PathData, Vec<Point>) {
        match self {
            Crumb::Line(line) => line.as_path_data_and_points(ts),
            Crumb::Rect(rect) => rect.as_path_data_and_points(ts),
            Crumb::RoundedRect(rr) => rr.as_path_data_and_points(ts),
            Crumb::Circle(circ) => circ.as_path_data_and_points(ts),
            Crumb::Arc(arc) => arc.as_path_data_and_points(ts),
            Crumb::Path(path) => path.as_path_data_and_points(ts),
        }
    }

    fn as_usvg_node_with_style(
        &self,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> (usvg::NodeKind, Vec<usvg::NodeKind>) {
        match self {
            Crumb::Line(line) => line.as_usvg_node_with_style(ts, style_id, theme),
            Crumb::Rect(rect) => rect.as_usvg_node_with_style(ts, style_id, theme),
            Crumb::RoundedRect(rr) => rr.as_usvg_node_with_style(ts, style_id, theme),
            Crumb::Circle(circ) => circ.as_usvg_node_with_style(ts, style_id, theme),
            Crumb::Arc(arc) => arc.as_usvg_node_with_style(ts, style_id, theme),
            Crumb::Path(path) => path.as_usvg_node_with_style(ts, style_id, theme),
        }
    }
}

impl AsUsvgNodeWithStyle for Line {
    fn end_angle(&self, _points: &[Point]) -> f64 {
        let angle = (self.p1.y - self.p0.y).atan2(self.p1.x - self.p0.x);

        if angle.is_nan() {
            0.0
        } else if angle < 0.0 {
            angle % (PI * 2.0) + PI * 2.0
        } else {
            angle % (PI * 2.0)
        }
    }

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

    fn as_usvg_node_with_style(
        &self,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> (usvg::NodeKind, Vec<usvg::NodeKind>) {
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

        (usvg::NodeKind::Path(usvg::Path { stroke, data, ..Default::default() }), more_kinds)
    }
}

impl AsUsvgNodeWithStyle for Rect {
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

impl AsUsvgNodeWithStyle for RoundedRect {
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

impl AsUsvgNodeWithStyle for Circle {
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

impl AsUsvgNodeWithStyle for Arc {
    fn as_path_data(&self, ts: TranslateScale) -> usvg::PathData {
        let path = BezPath::from_vec(self.path_elements(0.1).collect());

        path.as_path_data(ts)
    }
}

impl AsUsvgNodeWithStyle for BezPath {
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

    fn as_usvg_node_with_style(
        &self,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> (usvg::NodeKind, Vec<usvg::NodeKind>) {
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

        (usvg::NodeKind::Path(usvg::Path { stroke, fill, data, ..Default::default() }), more_kinds)
    }
}

pub trait AsUsvgNodeWithName {
    fn as_usvg_node_with_name<S: AsRef<str>>(&self, name: S) -> usvg::NodeKind;
}

impl AsUsvgNodeWithName for GradSpec {
    fn as_usvg_node_with_name<S: AsRef<str>>(&self, name: S) -> usvg::NodeKind {
        match self {
            GradSpec::Linear(start, end, stops) => {
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
            GradSpec::Radial(radius, stops) => {
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
                paint: usvg::Paint::Color(usvg::Color::new(red, green, blue)),
                width: self.get_width().into(),
                ..Default::default()
            }
        } else {
            usvg::Stroke {
                paint: usvg::Paint::Color(usvg::Color::new(red, green, blue)),
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

                (usvg::Paint::Color(usvg::Color::new(red, green, blue)), alpha)
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
            color:   usvg::Color::new(red, green, blue),
            opacity: if alpha == 0xff { 1.0.into() } else { (alpha as f64 / 255.0).into() },
        }
    }
}

pub struct BitmapDevice {
    rtree:     usvg::Tree,
    #[allow(dead_code)]
    transform: usvg::Transform,
    text:      NullText,
}

impl BitmapDevice {
    pub fn new(theme: &Theme, width: usize, height: usize, pix_scale: f64) -> Self {
        let svg_size = usvg::Size::new(width as f64, height as f64).unwrap();
        let mut rtree = usvg::Tree::create(usvg::Svg {
            size:     svg_size,
            view_box: usvg::ViewBox {
                rect:   svg_size.to_rect(0.0, 0.0),
                aspect: usvg::AspectRatio::default(),
            },
        });
        let transform = usvg::Transform::new_scale(pix_scale, pix_scale);

        for (name, spec) in theme.get_named_gradspecs() {
            let node = spec.as_usvg_node_with_name(name);

            rtree.append_to_defs(node);
        }

        theme.append_background_to_usvg_tree(&mut rtree);

        BitmapDevice { rtree, transform, text: NullText }
    }
}

impl std::fmt::Display for BitmapDevice {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.rtree.to_string(Default::default()))
    }
}

pub struct NullImage;

impl Image for NullImage {
    fn size(&self) -> Size {
        Size::ZERO
    }
}

#[derive(Clone)]
pub struct NullText;

impl Text for NullText {
    type TextLayout = NullTextLayout;
    type TextLayoutBuilder = NullTextLayoutBuilder;

    fn load_font(&mut self, _data: &[u8]) -> Result<FontFamily, Error> {
        Ok(FontFamily::default())
    }

    fn new_text_layout(&mut self, _text: impl TextStorage) -> Self::TextLayoutBuilder {
        NullTextLayoutBuilder
    }

    fn font_family(&mut self, _family_name: &str) -> Option<FontFamily> {
        Some(FontFamily::default())
    }
}

#[derive(Clone)]
pub struct NullTextLayout;

impl TextLayout for NullTextLayout {
    fn size(&self) -> Size {
        Size::ZERO
    }

    fn trailing_whitespace_width(&self) -> f64 {
        0.0
    }

    fn image_bounds(&self) -> Rect {
        Rect::ZERO
    }

    fn line_text(&self, _line_number: usize) -> Option<&str> {
        None
    }

    fn line_metric(&self, _line_number: usize) -> Option<LineMetric> {
        None
    }

    fn line_count(&self) -> usize {
        0
    }

    fn hit_test_point(&self, _point: Point) -> HitTestPoint {
        HitTestPoint::default()
    }

    fn hit_test_text_position(&self, _text_position: usize) -> HitTestPosition {
        HitTestPosition::default()
    }

    fn text(&self) -> &str {
        ""
    }
}

pub struct NullTextLayoutBuilder;

impl TextLayoutBuilder for NullTextLayoutBuilder {
    type Out = NullTextLayout;

    fn max_width(self, _width: f64) -> Self {
        self
    }

    fn alignment(self, _alignment: TextAlignment) -> Self {
        self
    }

    fn default_attribute(self, _attribute: impl Into<TextAttribute>) -> Self {
        self
    }

    fn range_attribute(
        self,
        _range: impl RangeBounds<usize>,
        _attribute: impl Into<TextAttribute>,
    ) -> Self {
        self
    }

    fn build(self) -> Result<Self::Out, Error> {
        Ok(NullTextLayout)
    }
}

impl IntoBrush<BitmapDevice> for Fill {
    fn make_brush<'b>(
        &'b self,
        _device: &mut BitmapDevice,
        _bbox: impl FnOnce() -> Rect,
    ) -> Cow<'b, Self> {
        Cow::Borrowed(self)
    }
}

impl RenderContext for BitmapDevice {
    type Brush = Fill;

    type Image = NullImage;
    type Text = NullText;
    type TextLayout = NullTextLayout;

    fn status(&mut self) -> Result<(), piet::Error> {
        Ok(())
    }

    fn solid_brush(&mut self, color: Color) -> Self::Brush {
        Fill::Color(color)
    }

    fn gradient(
        &mut self,
        _gradient: impl Into<FixedGradient>,
    ) -> Result<Self::Brush, piet::Error> {
        Ok(Fill::Color(Color::WHITE))
    }

    fn clear(&mut self, _region: impl Into<Option<Rect>>, _color: Color) {}

    fn stroke(&mut self, _shape: impl Shape, _brush: &impl IntoBrush<Self>, _width: f64) {}

    fn stroke_styled(
        &mut self,
        _shape: impl Shape,
        _brush: &impl IntoBrush<Self>,
        _width: f64,
        _style: &StrokeStyle,
    ) {
    }

    fn fill(&mut self, _shape: impl Shape, _brush: &impl IntoBrush<Self>) {}

    fn fill_even_odd(&mut self, _shape: impl Shape, _brush: &impl IntoBrush<Self>) {}

    fn clip(&mut self, _shape: impl Shape) {}

    fn text(&mut self) -> &mut Self::Text {
        &mut self.text
    }

    fn draw_text(&mut self, _layout: &Self::TextLayout, _pos: impl Into<Point>) {}

    fn save(&mut self) -> Result<(), piet::Error> {
        Ok(())
    }

    fn restore(&mut self) -> Result<(), piet::Error> {
        Ok(())
    }

    fn finish(&mut self) -> Result<(), piet::Error> {
        Ok(())
    }

    fn transform(&mut self, _transform: Affine) {}

    fn make_image(
        &mut self,
        _width: usize,
        _height: usize,
        _buf: &[u8],
        _format: ImageFormat,
    ) -> Result<Self::Image, piet::Error> {
        Ok(NullImage)
    }

    fn draw_image(
        &mut self,
        _image: &Self::Image,
        _dst_rect: impl Into<Rect>,
        _interp: InterpolationMode,
    ) {
    }

    fn draw_image_area(
        &mut self,
        _image: &Self::Image,
        _src_rect: impl Into<Rect>,
        _dst_rect: impl Into<Rect>,
        _interp: InterpolationMode,
    ) {
    }

    fn capture_image_area(&mut self, _src_rect: impl Into<Rect>) -> Result<Self::Image, Error> {
        Ok(NullImage)
    }

    fn blurred_rect(&mut self, _rect: Rect, _blur_radius: f64, _brush: &impl IntoBrush<Self>) {}

    fn current_transform(&self) -> Affine {
        Affine::default()
    }
}
