use std::{borrow::Cow, ops::RangeBounds};
use kurbo::{Point, Rect, Shape, Size, Affine};
use piet::{
    Color, Error, FixedGradient, FontFamily, HitTestPoint, HitTestPosition, Image, ImageFormat,
    InterpolationMode, IntoBrush, LineMetric, RenderContext, StrokeStyle, Text, TextAlignment,
    TextAttribute, TextLayout, TextLayoutBuilder, TextStorage,
};
use crate::{Theme, Fill, AsUsvgNodeWithName};

pub struct BitmapDevice {
    rtree: usvg::Tree,
    text:  NullText,
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

        for (name, spec) in theme.get_named_gradspecs() {
            let node = spec.as_usvg_node_with_name(name);

            rtree.append_to_defs(node);
        }

        theme.append_background_to_usvg_tree(&mut rtree);

        BitmapDevice { rtree, text: NullText }
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

    fn gradient(&mut self, gradient: impl Into<FixedGradient>) -> Result<Self::Brush, piet::Error> {
        Ok(Fill::Color(Color::WHITE))
    }

    fn clear(&mut self, color: Color) {}

    fn stroke(&mut self, shape: impl Shape, brush: &impl IntoBrush<Self>, width: f64) {}

    fn stroke_styled(
        &mut self,
        shape: impl Shape,
        brush: &impl IntoBrush<Self>,
        width: f64,
        style: &StrokeStyle,
    ) {
    }

    fn fill(&mut self, shape: impl Shape, brush: &impl IntoBrush<Self>) {}

    fn fill_even_odd(&mut self, shape: impl Shape, brush: &impl IntoBrush<Self>) {}

    fn clip(&mut self, shape: impl Shape) {}

    fn text(&mut self) -> &mut Self::Text {
        &mut self.text
    }

    fn draw_text(&mut self, layout: &Self::TextLayout, pos: impl Into<Point>) {}

    fn save(&mut self) -> Result<(), piet::Error> {
        Ok(())
    }

    fn restore(&mut self) -> Result<(), piet::Error> {
        Ok(())
    }

    fn finish(&mut self) -> Result<(), piet::Error> {
        Ok(())
    }

    fn transform(&mut self, transform: Affine) {}

    fn make_image(
        &mut self,
        width: usize,
        height: usize,
        buf: &[u8],
        format: ImageFormat,
    ) -> Result<Self::Image, piet::Error> {
        Ok(NullImage)
    }

    fn draw_image(
        &mut self,
        image: &Self::Image,
        dst_rect: impl Into<Rect>,
        interp: InterpolationMode,
    ) {
    }

    fn draw_image_area(
        &mut self,
        image: &Self::Image,
        src_rect: impl Into<Rect>,
        dst_rect: impl Into<Rect>,
        interp: InterpolationMode,
    ) {
    }

    fn blurred_rect(&mut self, rect: Rect, blur_radius: f64, brush: &impl IntoBrush<Self>) {}

    fn current_transform(&self) -> Affine {
        Affine::default()
    }
}
