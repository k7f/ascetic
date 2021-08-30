use crate::AsCss;

#[derive(Clone, Debug)]
pub struct Font {
    names:  Vec<String>,
    class:  GenericFontFamily,
    size:   f64, // point size (`pt` in svg)
    weight: FontWeight,
    style:  FontStyle,
}

impl Font {
    pub const DEFAULT_SIZE: f64 = 12.0;

    pub fn new<S: AsRef<str>>(name: S) -> Self {
        Font {
            names:  vec![name.as_ref().to_string()],
            class:  GenericFontFamily::Unspecified,
            size:   Self::DEFAULT_SIZE,
            weight: FontWeight::Normal,
            style:  FontStyle::Normal,
        }
    }

    pub const fn new_serif() -> Self {
        Font {
            names:  Vec::new(),
            class:  GenericFontFamily::Serif,
            size:   Self::DEFAULT_SIZE,
            weight: FontWeight::Normal,
            style:  FontStyle::Normal,
        }
    }

    pub const fn new_sans_serif() -> Self {
        Font {
            names:  Vec::new(),
            class:  GenericFontFamily::SansSerif,
            size:   Self::DEFAULT_SIZE,
            weight: FontWeight::Normal,
            style:  FontStyle::Normal,
        }
    }

    pub const fn new_cursive() -> Self {
        Font {
            names:  Vec::new(),
            class:  GenericFontFamily::Cursive,
            size:   Self::DEFAULT_SIZE,
            weight: FontWeight::Normal,
            style:  FontStyle::Normal,
        }
    }

    pub const fn new_monospace() -> Self {
        Font {
            names:  Vec::new(),
            class:  GenericFontFamily::Monospace,
            size:   Self::DEFAULT_SIZE,
            weight: FontWeight::Normal,
            style:  FontStyle::Normal,
        }
    }

    pub fn with_names<N, I>(mut self, names: I) -> Self
    where
        N: AsRef<str>,
        I: IntoIterator<Item = N>,
    {
        self.append_names(names);
        self
    }

    pub fn with_size(mut self, size: f64) -> Self {
        self.set_size(size);
        self
    }

    pub fn with_weight(mut self, value: u16) -> Self {
        self.set_weight(value);
        self
    }

    pub fn with_bold_weight(mut self) -> Self {
        self.set_bold_weight();
        self
    }

    pub fn with_italic_style(mut self) -> Self {
        self.set_italic_style();
        self
    }

    pub fn with_oblique_style(mut self) -> Self {
        self.set_oblique_style();
        self
    }

    pub fn append_names<N, I>(&mut self, names: I)
    where
        N: AsRef<str>,
        I: IntoIterator<Item = N>,
    {
        self.names.extend(names.into_iter().map(|name| name.as_ref().to_string()));
    }

    pub fn set_size(&mut self, size: f64) {
        self.size = size;
    }

    pub fn set_weight(&mut self, value: u16) {
        self.weight = FontWeight::Number(value.clamp(1, 1000));
    }

    pub fn set_bold_weight(&mut self) {
        self.weight = FontWeight::Bold;
    }

    pub fn set_italic_style(&mut self) {
        self.style = FontStyle::Italic;
    }

    pub fn set_oblique_style(&mut self) {
        self.style = FontStyle::Oblique;
    }

    #[inline]
    pub fn get_family_name(&self) -> &str {
        if let Some(family) = self.names.first() {
            family.as_str()
        } else {
            match &self.class {
                GenericFontFamily::Unspecified => unreachable!(),
                class => class.as_css(),
            }
        }
    }

    #[inline]
    pub fn get_size(&self) -> f64 {
        self.size
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GenericFontFamily {
    Serif,
    SansSerif,
    Cursive,
    Monospace,
    Unspecified,
}

impl AsCss for GenericFontFamily {
    fn as_css(&self) -> &str {
        use GenericFontFamily::*;

        match self {
            Serif => "serif",
            SansSerif => "sans-serif",
            Cursive => "cursive",
            Monospace => "monospace",
            Unspecified => "",
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum FontWeight {
    Normal,
    Bold,
    Number(u16),
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
