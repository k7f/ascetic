#[derive(Clone, Debug)]
pub struct Font {
    family: Vec<FontFamily>,
    size:   f64, // point size (`pt` in svg)
    weight: FontWeight,
    style:  FontStyle,
}

impl Default for Font {
    fn default() -> Self {
        Font {
            family: vec![FontFamily::default()],
            size:   12.0,
            weight: FontWeight::default(),
            style:  FontStyle::default(),
        }
    }
}

impl Font {
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
}

#[derive(Clone, Hash, Debug)]
pub(crate) enum GenericFontFamily {
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

#[derive(Clone, Debug)]
enum FontFamily {
    Generic(GenericFontFamily),
    Specific(String),
}

impl Default for FontFamily {
    #[inline]
    fn default() -> Self {
        FontFamily::Generic(Default::default())
    }
}

#[derive(Clone, Debug)]
enum FontWeight {
    Normal,
    Bold,
    Number(f64),
}

impl Default for FontWeight {
    #[inline]
    fn default() -> Self {
        FontWeight::Normal
    }
}

#[derive(Clone, Debug)]
enum FontStyle {
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
