use std::{borrow::Cow, ops::RangeBounds};
use kurbo::{Point, Line, Rect, RoundedRect, Circle, Shape, TranslateScale, Size, Affine};
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

        for (name, spec) in theme.get_named_gradspecs() {
            let node = spec.as_usvg_node_with_name(name);

            rtree.append_to_defs(node);
        }

        theme.append_background_to_usvg_tree(&mut rtree);

        for CrumbItem(crumb_id, ts, style_id) in self.all_crumbs(root_ts) {
            if let Some(crumb) = self.get_crumb(crumb_id) {
                let node = crumb.as_usvg_node_with_style(ts, style_id, theme);
                rtree.root().append_kind(node);
            } else {
                // FIXME
                panic!()
            }
        }

        rtree
    }
}

pub trait AsUsvgNodeWithStyle {
    fn as_usvg_node_with_style(
        &self,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> usvg::NodeKind;
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

fn rect_data(rect: Rect) -> usvg::PathData {
    usvg::PathData(vec![
        usvg::PathSegment::MoveTo { x: rect.x0, y: rect.y0 },
        usvg::PathSegment::LineTo { x: rect.x1, y: rect.y0 },
        usvg::PathSegment::LineTo { x: rect.x1, y: rect.y1 },
        usvg::PathSegment::LineTo { x: rect.x0, y: rect.y1 },
        usvg::PathSegment::ClosePath,
    ])
}

impl AsUsvgNodeWithStyle for Rect {
    fn as_usvg_node_with_style(
        &self,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> usvg::NodeKind {
        let (fill, stroke) = theme.get_style_as_usvg(style_id);
        let data = std::rc::Rc::new(rect_data(ts * *self));

        usvg::NodeKind::Path(usvg::Path { fill, stroke, data, ..Default::default() })
    }
}

fn rr_data(rect: Rect, rx: f64, ry: f64) -> usvg::PathData {
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
}

impl AsUsvgNodeWithStyle for RoundedRect {
    fn as_usvg_node_with_style(
        &self,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> usvg::NodeKind {
        let (fill, stroke) = theme.get_style_as_usvg(style_id);
        let rr = ts * *self;
        let data = std::rc::Rc::new(if let Some(radius) = rr.radii().as_single_radius() {
            rr_data(rr.rect(), radius, radius)
        } else {
            rect_data(rr.rect())
        });

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
