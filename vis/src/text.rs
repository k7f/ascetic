use crate::{Theme, Style, Font, GenericFontFamily};

#[derive(Clone, Debug)]
pub struct TextLabel {
    pub(crate) x:    f64,
    pub(crate) y:    f64,
    pub(crate) body: String,
    pub(crate) font: Option<Font>,
}

impl TextLabel {
    pub const DEFAULT_FONT: Font = Font::new();

    // fn new(body: S, rc: &mut R, style: Option<&Style>, theme: &Theme)

    pub fn new(body: String) -> Self {
        TextLabel { x: 0.0, y: 0.0, body, font: None }
    }

    pub fn with_xy(mut self, x: f64, y: f64) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn with_font(mut self, font: Font) -> Self {
        self.set_font(font);
        self
    }

    pub fn set_font(&mut self, font: Font) {
        self.font = Some(font);
    }

    pub fn get_font(&mut self) -> Option<&Font> {
        self.font.as_ref()
    }

    pub fn resolve_font<'a, 'b: 'a, 'c: 'a>(
        &'a self,
        style: Option<&'b Style>,
        theme: &'c Theme,
    ) -> Option<&'a Font> {
        self.font.as_ref().or_else(|| {
            style
                .and_then(|s| s.get_font())
                .or_else(|| theme.get_generic_font(GenericFontFamily::default()))
        })
    }
}
