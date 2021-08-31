use std::{borrow::Cow, ops::RangeBounds};
use kurbo::{Point, Rect, Shape, TranslateScale, Size, Affine};
use piet::{
    Color, Error, FixedGradient, FontFamily, HitTestPoint, HitTestPosition, Image, ImageFormat,
    InterpolationMode, IntoBrush, LineMetric, RenderContext, StrokeStyle, Text, TextAlignment,
    TextAttribute, TextLayout, TextLayoutBuilder, TextStorage,
};
use crate::{Vis, Scene, Theme, Style, Fill, Crumb, CrumbItem, TextLabel};

pub struct XmlDevice {
    #[allow(dead_code)]
    out_size: Size,
    #[allow(dead_code)]
    ts:       TranslateScale,
    text:     NullText,
}

impl XmlDevice {
    pub fn new(_theme: &Theme, width: usize, height: usize, pix_scale: f64) -> Self {
        let out_size = Size::new(width as f64, height as f64);
        let ts = TranslateScale::scale(pix_scale);

        XmlDevice { out_size, ts, text: NullText }
    }
}

impl std::fmt::Display for XmlDevice {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "FIXME")
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

impl IntoBrush<XmlDevice> for Fill {
    fn make_brush<'b>(
        &'b self,
        _device: &mut XmlDevice,
        _bbox: impl FnOnce() -> Rect,
    ) -> Cow<'b, Self> {
        Cow::Borrowed(self)
    }
}

impl RenderContext for XmlDevice {
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

impl Scene {
    pub fn render_svg<S, M>(
        &self,
        theme: &Theme,
        out_size: S,
        out_margin: M,
        rc: &mut XmlDevice,
    ) -> Result<(), Box<dyn std::error::Error>>
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

        rc.clear(None, theme.get_bg_color());

        for CrumbItem(crumb_id, ts, style_id) in self.all_crumbs(root_ts) {
            if let Some(crumb) = self.get_crumb(crumb_id) {
                let style = theme.get_style(style_id);

                crumb.vis(rc, ts, style, theme);
            } else {
                // FIXME
                panic!()
            }
        }

        rc.finish()?;

        Ok(())
    }
}

impl Vis<XmlDevice> for Crumb {
    fn bbox(&self, rc: &mut XmlDevice, ts: TranslateScale) -> Rect {
        match self {
            Crumb::Line(line) => line.bbox(rc, ts),
            Crumb::Rect(rect) => rect.bbox(rc, ts),
            Crumb::RoundedRect(rr) => rr.bbox(rc, ts),
            Crumb::Circle(circ) => circ.bbox(rc, ts),
            Crumb::Arc(arc) => arc.bbox(rc, ts),
            Crumb::Path(path) => path.bbox(rc, ts),
            Crumb::Pin(pin) => pin.bbox(rc, ts),
            Crumb::Label(label) => label.bbox(rc, ts),
        }
    }

    fn vis(&self, rc: &mut XmlDevice, ts: TranslateScale, style: Option<&Style>, theme: &Theme) {
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

// FIXME cache text_layout somehow
impl Vis<XmlDevice> for TextLabel {
    fn bbox(&self, rc: &mut XmlDevice, ts: TranslateScale) -> Rect {
        if let Some(body) = self.get_body().first() {
            if let crate::text::Item::Text(body) = body {
                let text = rc.text();
                let font = FontFamily::SANS_SERIF;

                if let Ok(layout) = text.new_text_layout(body.clone()).font(font, 12.0).build() {
                    return ts * layout.image_bounds() // FIXME translate to self.origin
                }
            }
        }

        // FIXME
        let origin = self.get_origin().unwrap_or_default();
        ts * Rect::new(origin.x, origin.y, origin.x + 100.0, origin.y + 30.0)
    }

    fn vis(
        &self,
        _rc: &mut XmlDevice,
        _ts: TranslateScale,
        _style: Option<&Style>,
        _theme: &Theme,
    ) {
        unreachable!()
    }

    fn vis_mut(
        &mut self,
        rc: &mut XmlDevice,
        ts: TranslateScale,
        style: Option<&Style>,
        theme: &Theme,
    ) {
        self.resolve_font(style, theme);

        let rc_text = rc.text();
        let (font_name, font_size) = if let Some(font) = self.get_font() {
            (font.get_family_name(), font.get_size())
        } else {
            ("sans-serif", Self::DEFAULT_FONT.get_size())
        };
        let rc_font = rc_text.font_family(font_name).unwrap_or(FontFamily::SANS_SERIF);

        // FIXME draw all items
        if let Some(body) = self.get_body().first() {
            if let crate::text::Item::Text(body) = body {
                if let Ok(mut layout) =
                    rc_text.new_text_layout(body.clone()).font(rc_font, font_size).build()
                {
                    // FIXME
                    let origin = self.get_origin().unwrap_or_default();
                    rc.draw_text(&mut layout, ts * origin);
                }
            }
        }
    }
}
