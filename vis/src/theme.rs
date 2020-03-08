use std::collections::{HashMap, hash_map};
use piet_common::{Color, LinearGradient, RadialGradient, UnitPoint, GradientStops};
use crate::{Style, StyleId, Stroke, GradSpec};

#[derive(Default, Debug)]
pub struct Theme {
    default_style:    Style,
    scene_style:      Style,
    named_styles:     HashMap<String, StyleId>,
    styles:           Vec<Style>,
    named_gradspecs:  HashMap<String, GradSpec>,
    linear_gradients: HashMap<String, LinearGradient>,
    radial_gradients: HashMap<String, RadialGradient>,
}

impl Theme {
    pub fn new() -> Self {
        Theme::default()
    }

    pub fn with_default_style(mut self, default_style: Style) -> Self {
        self.default_style = default_style;
        self
    }

    pub fn with_scene_style(mut self, scene_style: Style) -> Self {
        self.scene_style = scene_style;
        self
    }

    pub fn with_named_styles<S, I>(mut self, styles: I) -> Self
    where
        S: AsRef<str>,
        I: IntoIterator<Item = (S, Style)>,
    {
        for (name, style) in styles.into_iter() {
            let id = self.styles.len();

            self.styles.push(style);
            self.named_styles.insert(name.as_ref().into(), StyleId(id));
        }

        self
    }

    pub fn with_gradients<S, G, I, J>(mut self, linear_gradients: I, radial_gradients: J) -> Self
    where
        S: AsRef<str>,
        G: GradientStops,
        I: IntoIterator<Item = (S, UnitPoint, UnitPoint, G)>,
        J: IntoIterator<Item = (S, f64, G)>,
    {
        for (name, start, end, stops) in linear_gradients.into_iter() {
            let stops = stops.to_vec();
            let gradient = LinearGradient::new(start, end, stops.clone());

            self.named_gradspecs.insert(name.as_ref().into(), GradSpec::Linear(start, end, stops));
            self.linear_gradients.insert(name.as_ref().into(), gradient);
        }

        for (name, radius, stops) in radial_gradients.into_iter() {
            let stops = stops.to_vec();
            let gradient = RadialGradient::new(radius, stops.clone());

            self.named_gradspecs.insert(name.as_ref().into(), GradSpec::Radial(radius, stops));
            self.radial_gradients.insert(name.as_ref().into(), gradient);
        }

        self
    }

    #[inline]
    pub fn get<S: AsRef<str>>(&self, name: S) -> Option<StyleId> {
        self.named_styles.get(name.as_ref()).copied()
    }

    #[inline]
    pub fn get_default_style(&self) -> Option<&Style> {
        Some(&self.default_style)
    }

    #[inline]
    pub fn get_default_stroke(&self) -> Option<&Stroke> {
        self.default_style.get_stroke()
    }

    #[inline]
    pub fn get_style(&self, style_id: Option<StyleId>) -> Option<&Style> {
        style_id.and_then(|id| self.styles.get(id.0))
    }

    #[inline]
    pub fn get_stroke(&self, style_id: Option<StyleId>) -> Option<&Stroke> {
        self.get_style(style_id).and_then(|s| s.get_stroke())
    }

    #[inline]
    pub fn get_linear_gradient<S: AsRef<str>>(&self, name: S) -> Option<&LinearGradient> {
        self.linear_gradients.get(name.as_ref())
    }

    #[inline]
    pub fn get_radial_gradient<S: AsRef<str>>(&self, name: S) -> Option<&RadialGradient> {
        self.radial_gradients.get(name.as_ref())
    }

    #[inline]
    pub fn get_gradspec<S: AsRef<str>>(&self, name: S) -> Option<&GradSpec> {
        self.named_gradspecs.get(name.as_ref())
    }

    #[inline]
    pub fn get_bg_color(&self) -> Color {
        self.scene_style.get_fill_color().cloned().unwrap_or(Color::WHITE)
    }

    #[inline]
    pub fn get_linear_gradients(&self) -> hash_map::Iter<String, LinearGradient> {
        self.linear_gradients.iter()
    }

    #[inline]
    pub fn get_radial_gradients(&self) -> hash_map::Iter<String, RadialGradient> {
        self.radial_gradients.iter()
    }

    #[inline]
    pub fn get_named_gradspecs(&self) -> hash_map::Iter<String, GradSpec> {
        self.named_gradspecs.iter()
    }
}
