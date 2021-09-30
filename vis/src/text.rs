use kurbo::{Point, TranslateScale};
use crate::{Theme, Style, Font, PreprocessWithStyle, VisError};

#[derive(Clone, Copy, Debug)]
pub(crate) enum Anchor {
    Start,
    Middle,
    End,
}

#[derive(Clone, Debug)]
pub(crate) enum Item {
    Text(String),
    Span(TextLabel),
}

#[derive(Clone, Debug)]
pub struct TextLabel {
    origin:           Option<Point>,
    dx:               Vec<f64>,
    dy:               Vec<f64>,
    anchor:           Anchor,
    body:             Vec<Item>,
    font:             Option<Font>,
    is_root:          bool,
    font_is_explicit: bool,
    font_size:        Option<f64>,
}

impl TextLabel {
    pub const DEFAULT_FONT: Font = Font::new_sans_serif();

    pub fn new() -> Self {
        TextLabel {
            origin:           None,
            dx:               Vec::new(),
            dy:               Vec::new(),
            anchor:           Anchor::Start,
            body:             Vec::new(),
            font:             None,
            is_root:          true,
            font_is_explicit: false,
            font_size:        None,
        }
    }

    pub fn with_text<S: AsRef<str>>(mut self, body: S) -> Self {
        self.body.push(Item::Text(body.as_ref().to_string()));
        self
    }

    pub fn with_span(mut self, mut span: Self) -> Self {
        span.is_root = false;
        self.body.push(Item::Span(span));
        self
    }

    pub fn with_origin<P: Into<Point>>(mut self, origin: P) -> Self {
        self.set_origin(origin);
        self
    }

    pub fn with_xy(mut self, x: f64, y: f64) -> Self {
        self.set_xy(x, y);
        self
    }

    pub fn with_dx<I>(mut self, dx: I) -> Self
    where
        I: IntoIterator<Item = f64>,
    {
        self.set_dx(dx);
        self
    }

    pub fn with_dy<I>(mut self, dy: I) -> Self
    where
        I: IntoIterator<Item = f64>,
    {
        self.set_dy(dy);
        self
    }

    pub fn with_middle_anchor(mut self) -> Self {
        self.set_middle_anchor();
        self
    }

    pub fn with_end_anchor(mut self) -> Self {
        self.set_end_anchor();
        self
    }

    pub fn with_font(mut self, font: Font) -> Self {
        self.set_font(font);
        self
    }

    pub fn with_font_size(mut self, font_size: f64) -> Self {
        self.set_font_size(font_size);
        self
    }

    pub fn append_text<S: AsRef<str>>(&mut self, body: S) {
        self.body.push(Item::Text(body.as_ref().to_string()));
    }

    pub fn append_span(&mut self, mut span: Self) {
        span.is_root = false;
        self.body.push(Item::Span(span));
    }

    pub fn set_origin<P: Into<Point>>(&mut self, origin: P) {
        self.origin = Some(origin.into());
    }

    pub fn set_xy(&mut self, x: f64, y: f64) {
        self.origin = Some(Point::new(x, y));
    }

    pub fn set_dx<I>(&mut self, dx: I)
    where
        I: IntoIterator<Item = f64>,
    {
        self.dx = dx.into_iter().collect();
    }

    pub fn set_dy<I>(&mut self, dy: I)
    where
        I: IntoIterator<Item = f64>,
    {
        self.dy = dy.into_iter().collect();
    }

    pub fn set_middle_anchor(&mut self) {
        self.anchor = Anchor::Middle;
    }

    pub fn set_end_anchor(&mut self) {
        self.anchor = Anchor::End;
    }

    pub fn set_font(&mut self, font: Font) {
        self.font = Some(font);
        self.font_is_explicit = true;
    }

    pub fn set_font_size(&mut self, font_size: f64) {
        if let Some(ref mut font) = self.font {
            font.set_size(font_size);
        }
        self.font_size = Some(font_size);
    }

    #[inline]
    pub fn is_root(&self) -> bool {
        self.is_root
    }

    #[inline]
    pub fn get_origin(&self) -> Option<Point> {
        self.origin
    }

    #[inline]
    pub fn get_dx(&self) -> &[f64] {
        self.dx.as_slice()
    }

    #[inline]
    pub fn get_dy(&self) -> &[f64] {
        self.dy.as_slice()
    }

    #[inline]
    pub(crate) fn get_anchor(&self) -> Anchor {
        self.anchor
    }

    #[inline]
    pub(crate) fn get_body(&self) -> &[Item] {
        self.body.as_slice()
    }

    #[inline]
    pub(crate) fn get_body_mut(&mut self) -> &mut [Item] {
        self.body.as_mut_slice()
    }

    #[inline]
    pub fn get_font(&self) -> Option<&Font> {
        self.font.as_ref()
    }

    #[inline]
    pub fn get_font_size(&self) -> Option<f64> {
        if let Some(ref font) = self.font {
            Some(font.get_size())
        } else {
            self.font_size
        }
    }

    pub fn resolve_font(&mut self, style: Option<&Style>, theme: &Theme) {
        if self.font_is_explicit {
            assert!(self.font.is_some())
        } else {
            let mut font = style
                .and_then(|s| s.get_font())
                .or_else(|| theme.get_sans_serif_font())
                .unwrap_or(&Self::DEFAULT_FONT)
                .clone();

            if let Some(size) = self.font_size {
                font.set_size(size);
            }

            self.font = Some(font);
        }
    }
}

impl Default for TextLabel {
    fn default() -> Self {
        Self::new()
    }
}

impl PreprocessWithStyle for TextLabel {
    fn preprocess_with_style(
        &mut self,
        ts: TranslateScale,
        style: Option<&Style>,
        theme: &Theme,
    ) -> Result<(), VisError> {
        let resolved_style = Some(style.unwrap_or_else(|| theme.get_default_style()));

        self.resolve_font(resolved_style, theme);

        for item in self.get_body_mut() {
            match item {
                crate::text::Item::Text(_) => {}
                crate::text::Item::Span(span) => {
                    span.preprocess_with_style(ts, style, theme)?;
                }
            }
        }

        Ok(())
    }
}
