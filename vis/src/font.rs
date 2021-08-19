use crate::AsCss;

#[derive(Clone, Debug)]
pub struct Font {
    pub(crate) family: Vec<FontFamily>,
    size:   f64, // point size (`pt` in svg)
    weight: FontWeight,
    style:  FontStyle,
}

impl Default for Font {
    #[inline]
    fn default() -> Self {
        Font::new()
    }
}

impl Font {
    pub const fn new() -> Self {
        Font {
            family: Vec::new(),
            size:   12.0,
            weight: FontWeight::Normal,
            style:  FontStyle::Normal,
        }
    }

    pub fn new_serif() -> Self {
        Font { family: vec![FontFamily::Generic(GenericFontFamily::Serif)], ..Default::default() }
    }

    pub fn new_sans_serif() -> Self {
        Font {
            family: vec![FontFamily::Generic(GenericFontFamily::SansSerif)],
            ..Default::default()
        }
    }

    pub fn with_family<N, I>(mut self, family: I) -> Self
    where
        N: AsRef<str>,
        I: IntoIterator<Item = N>,
    {
        for (pos, name) in family.into_iter().enumerate() {
            let name = name.as_ref().to_string();

            self.family.insert(pos, FontFamily::Specific(name));
        }
        self
    }

    pub fn with_size(mut self, size: f64) -> Self {
        self.size = size;
        self
    }

    pub fn with_weight(mut self, value: u16) -> Self {
        self.weight = FontWeight::Number(value.clamp(1, 1000));
        self
    }

    pub fn with_bold_weight(mut self) -> Self {
        self.weight = FontWeight::Bold;
        self
    }

    pub fn with_italic_style(mut self) -> Self {
        self.style = FontStyle::Italic;
        self
    }

    pub fn with_oblique_style(mut self) -> Self {
        self.style = FontStyle::Oblique;
        self
    }

    #[inline]
    pub fn get_family_name(&self) -> &str {
        if let Some(family) = self.family.first() {
            family.as_css()
        } else {
            "Times"
        }
    }

    #[inline]
    pub fn get_size(&self) -> f64 {
        self.size
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum GenericFontFamily {
    Serif,
    SansSerif,
    Cursive,
    Monospace,
}

impl Default for GenericFontFamily {
    #[inline]
    fn default() -> Self {
        GenericFontFamily::Serif
    }
}

impl AsCss for GenericFontFamily {
    fn as_css(&self) -> &str {
        use GenericFontFamily::*;

        match self {
            Serif => "serif",
            SansSerif => "sans-serif",
            Cursive => "cursive",
            Monospace => "monospace",
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum FontFamily {
    Generic(GenericFontFamily),
    Specific(String),
}

impl Default for FontFamily {
    #[inline]
    fn default() -> Self {
        FontFamily::Generic(Default::default())
    }
}

impl AsCss for FontFamily {
    fn as_css(&self) -> &str {
        use FontFamily::*;

        match self {
            Generic(form) => form.as_css(),
            Specific(name) => name,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum FontWeight {
    Normal,
    Bold,
    Number(u16),
}

impl Default for FontWeight {
    #[inline]
    fn default() -> Self {
        FontWeight::Normal
    }
}

impl FontWeight {
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn as_number(&self) -> u16 {
        use FontWeight::*;

        match *self {
            Normal => 400,
            Bold => 700,
            Number(v) => v,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

impl Default for FontStyle {
    #[inline]
    fn default() -> Self {
        FontStyle::Normal
    }
}
