use std::{
    collections::{HashMap, hash_map},
    iter::FromIterator,
};
use crate::{
    Style, StyleId, Color, Stroke, Fill, UnitPoint, GradientStops, Gradient, Marker, MarkerId,
    Font, font::GenericFontFamily, VisError,
};

const DEFAULT_NAME: &str = "default";
const SCENE_NAME: &str = "scene";

#[derive(Debug)]
pub struct Variation {
    strokes:    HashMap<String, Stroke>,
    fills:      HashMap<String, Fill>,
    variations: HashMap<String, Variation>,
}

impl Default for Variation {
    fn default() -> Self {
        let strokes = HashMap::from_iter(vec![
            (DEFAULT_NAME.into(), Stroke::default()),
            (SCENE_NAME.into(), Stroke::default()),
        ]);
        let fills = HashMap::from_iter(vec![
            (DEFAULT_NAME.into(), Fill::default()),
            (SCENE_NAME.into(), Fill::default()),
        ]);
        let variations = HashMap::default();

        Variation { strokes, fills, variations }
    }
}

impl Variation {
    pub fn new() -> Self {
        Variation::default()
    }

    pub fn with_strokes<S, I>(mut self, strokes: I) -> Self
    where
        S: AsRef<str>,
        I: IntoIterator<Item = (S, Stroke)>,
    {
        self.add_strokes(strokes);

        self
    }

    pub fn with_fills<S, I>(mut self, fills: I) -> Self
    where
        S: AsRef<str>,
        I: IntoIterator<Item = (S, Fill)>,
    {
        self.add_fills(fills);

        self
    }

    pub fn add_strokes<S, I>(&mut self, strokes: I)
    where
        S: AsRef<str>,
        I: IntoIterator<Item = (S, Stroke)>,
    {
        for (name, stroke) in strokes.into_iter() {
            self.strokes.insert(name.as_ref().into(), stroke);
        }
    }

    pub fn add_fills<S, I>(&mut self, fills: I)
    where
        S: AsRef<str>,
        I: IntoIterator<Item = (S, Fill)>,
    {
        for (name, fill) in fills.into_iter() {
            self.fills.insert(name.as_ref().into(), fill);
        }
    }

    #[inline]
    pub fn get_stroke_by_name<S: AsRef<str>>(&self, name: S) -> Option<&Stroke> {
        self.strokes.get(name.as_ref())
    }

    #[inline]
    pub fn get_fill_by_name<S: AsRef<str>>(&self, name: S) -> Option<&Fill> {
        self.fills.get(name.as_ref())
    }

    pub fn get_stroke_by_path<V, I, S>(&self, path: I, name: S) -> Option<&Stroke>
    where
        V: AsRef<str>,
        I: IntoIterator<Item = V>,
        S: AsRef<str>,
    {
        let name = name.as_ref();
        let mut result = self.get_stroke_by_name(name);
        let mut variation = self;

        for nv in path.into_iter() {
            if let Some(v) = variation.variations.get(nv.as_ref()) {
                if let Some(s) = v.get_stroke_by_name(name) {
                    result = Some(s);
                }
                variation = v;
            } else {
                break
            }
        }

        result
    }

    pub fn get_fill_by_path<V, I, S>(&self, path: I, name: S) -> Option<&Fill>
    where
        V: AsRef<str>,
        I: IntoIterator<Item = V>,
        S: AsRef<str>,
    {
        let name = name.as_ref();
        let mut result = self.get_fill_by_name(name);
        let mut variation = self;

        for nv in path.into_iter() {
            if let Some(v) = variation.variations.get(nv.as_ref()) {
                if let Some(s) = v.get_fill_by_name(name) {
                    result = Some(s);
                }
                variation = v;
            } else {
                break
            }
        }

        result
    }
}

#[derive(Debug)]
pub struct Theme {
    original:        Variation,
    styles:          Vec<Style>,
    markers:         Vec<Marker>,
    named_styles:    HashMap<String, StyleId>,
    named_markers:   HashMap<String, MarkerId>,
    named_gradspecs: HashMap<String, Gradient>,
    default_fonts:   HashMap<GenericFontFamily, Font>,
}

impl Default for Theme {
    fn default() -> Self {
        let original = Variation::default();
        let styles = vec![
            Style::default().with_named_stroke(DEFAULT_NAME).with_named_fill(DEFAULT_NAME),
            Style::default().with_named_stroke(SCENE_NAME).with_named_fill(SCENE_NAME),
        ];
        let markers = Vec::new();
        let named_styles = HashMap::from_iter(vec![
            (DEFAULT_NAME.into(), Self::DEFAULT_STYLE_ID),
            (SCENE_NAME.into(), Self::SCENE_STYLE_ID),
        ]);
        let named_markers = HashMap::default();
        let named_gradspecs = HashMap::default();
        let default_fonts = HashMap::default();

        Theme {
            original,
            styles,
            markers,
            named_styles,
            named_markers,
            named_gradspecs,
            default_fonts,
        }
    }
}

impl Theme {
    const DEFAULT_STYLE_ID: StyleId = StyleId(0);
    const SCENE_STYLE_ID: StyleId = StyleId(1);

    pub fn new() -> Self {
        Theme::default()
    }

    /// Note: calling this is the only way of adding styles to a
    /// theme.
    pub fn with_styles<S, I>(mut self, styles: I) -> Self
    where
        S: AsRef<str>,
        I: IntoIterator<Item = (S, Style)>,
    {
        for (name, mut style) in styles.into_iter() {
            style.resolve_initially(&self.original);

            match self.named_styles.entry(name.as_ref().into()) {
                hash_map::Entry::Occupied(entry) => {
                    self.styles[entry.get().0] = style;
                }
                hash_map::Entry::Vacant(entry) => {
                    let id = self.styles.len();

                    entry.insert(StyleId(id));
                    self.styles.push(style);
                }
            }
        }

        self
    }

    pub fn with_variations<S, I>(mut self, variations: I) -> Self
    where
        S: AsRef<str>,
        I: IntoIterator<Item = (S, Variation)>,
    {
        for (name, variation) in variations.into_iter() {
            self.original.variations.insert(name.as_ref().into(), variation);
        }

        self
    }

    pub fn with_strokes<S, I>(mut self, strokes: I) -> Self
    where
        S: AsRef<str>,
        I: IntoIterator<Item = (S, Stroke)>,
    {
        self.original.add_strokes(strokes);

        for style in self.styles.iter_mut() {
            style.resolve_initially(&self.original);
        }

        self
    }

    pub fn with_fills<S, I>(mut self, fills: I) -> Self
    where
        S: AsRef<str>,
        I: IntoIterator<Item = (S, Fill)>,
    {
        self.original.add_fills(fills);

        for style in self.styles.iter_mut() {
            style.resolve_initially(&self.original);
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
            self.named_gradspecs
                .insert(name.as_ref().into(), Gradient::Linear(start, end, stops.into_vec()));
        }

        for (name, radius, stops) in radial_gradients.into_iter() {
            self.named_gradspecs
                .insert(name.as_ref().into(), Gradient::Radial(radius, stops.into_vec()));
        }

        for style in self.styles.iter_mut() {
            style.resolve_initially(&self.original);
        }

        self
    }

    /// Note: calling this is the only way of adding markers to a
    /// theme.
    pub fn with_markers<S, I>(mut self, markers: I) -> Self
    where
        S: AsRef<str>,
        I: IntoIterator<Item = (S, Marker)>,
    {
        for (name, marker) in markers.into_iter() {
            match self.named_markers.entry(name.as_ref().into()) {
                hash_map::Entry::Occupied(entry) => {
                    self.markers[entry.get().0] = marker;
                }
                hash_map::Entry::Vacant(entry) => {
                    let id = self.markers.len();

                    entry.insert(MarkerId(id));
                    self.markers.push(marker);
                }
            }
        }

        self
    }

    pub fn use_original_variation(&mut self) {
        for style in self.styles.iter_mut() {
            style.resolve_initially(&self.original);
        }
    }

    pub fn use_variation<V, I>(&mut self, path: I)
    where
        V: AsRef<str>,
        I: IntoIterator<Item = V> + Clone,
    {
        for style in self.styles.iter_mut() {
            style.resolve(&self.original, path.clone());
        }
    }

    pub fn start_original_variation(&mut self, max_subdivision: usize) {
        for style in self.styles.iter_mut() {
            style.start_original_resolution(&self.original, max_subdivision);
        }
    }

    pub fn start_variation<V, I>(&mut self, path: I, max_subdivision: usize)
    where
        V: AsRef<str>,
        I: IntoIterator<Item = V> + Clone,
    {
        for style in self.styles.iter_mut() {
            style.start_resolution(&self.original, path.clone(), max_subdivision);
        }
    }

    pub fn step_variation(&mut self, amount: f64) {
        for style in self.styles.iter_mut() {
            style.step_resolution(amount);
        }
    }

    #[inline]
    pub fn get<S: AsRef<str>>(&self, name: S) -> Option<StyleId> {
        self.named_styles.get(name.as_ref()).copied()
    }

    #[inline]
    pub fn get_default_style(&self) -> &Style {
        &self.styles[Self::DEFAULT_STYLE_ID.0]
    }

    #[inline]
    pub fn get_scene_style(&self) -> &Style {
        &self.styles[Self::SCENE_STYLE_ID.0]
    }

    #[inline]
    pub fn get_style(&self, style_id: Option<StyleId>) -> Option<&Style> {
        style_id.and_then(|id| self.styles.get(id.0))
    }

    #[inline]
    pub fn get_style_by_name<S: AsRef<str>>(&self, name: S) -> Option<&Style> {
        self.get(name).and_then(|id| self.styles.get(id.0))
    }

    #[inline]
    pub fn get_stroke(&self, style_id: Option<StyleId>) -> Option<&Stroke> {
        self.get_style(style_id).and_then(|s| s.get_stroke())
    }

    #[inline]
    pub fn get_stroke_by_name<S: AsRef<str>>(&self, name: S) -> Option<&Stroke> {
        self.original.get_stroke_by_name(name)
    }

    #[inline]
    pub fn get_stroke_by_path<V, I, S>(&self, path: I, name: S) -> Option<&Stroke>
    where
        V: AsRef<str>,
        I: IntoIterator<Item = V>,
        S: AsRef<str>,
    {
        self.original.get_stroke_by_path(path, name)
    }

    #[inline]
    pub fn get_fill(&self, style_id: Option<StyleId>) -> Option<&Fill> {
        self.get_style(style_id).and_then(|s| s.get_fill())
    }

    #[inline]
    pub fn get_fill_by_name<S: AsRef<str>>(&self, name: S) -> Option<&Fill> {
        self.original.get_fill_by_name(name)
    }

    #[inline]
    pub fn get_fill_by_path<V, I, S>(&self, path: I, name: S) -> Option<&Fill>
    where
        V: AsRef<str>,
        I: IntoIterator<Item = V>,
        S: AsRef<str>,
    {
        self.original.get_fill_by_path(path, name)
    }

    #[inline]
    pub fn get_gradspec<S: AsRef<str>>(&self, name: S) -> Option<&Gradient> {
        self.named_gradspecs.get(name.as_ref())
    }

    #[inline]
    pub fn get_marker(&self, marker_id: Option<MarkerId>) -> Option<&Marker> {
        marker_id.and_then(|id| self.markers.get(id.0))
    }

    #[inline]
    pub fn get_marker_mut(&mut self, marker_id: Option<MarkerId>) -> Option<&mut Marker> {
        marker_id.and_then(move |id| self.markers.get_mut(id.0))
    }

    #[inline]
    pub fn get_marker_by_name<S: AsRef<str>>(&self, name: S) -> Option<&Marker> {
        self.get_marker(self.named_markers.get(name.as_ref()).copied())
    }

    pub fn get_marker_width(&self, style_id: Option<StyleId>) -> (f64, f64) {
        self.get_style(style_id)
            .map(|style| style.get_markers())
            .map(|markers| {
                (
                    markers
                        .get_start_name()
                        .and_then(|name| self.get_marker_by_name(name))
                        .map(|marker| marker.get_width())
                        .unwrap_or(0.0),
                    markers
                        .get_end_name()
                        .and_then(|name| self.get_marker_by_name(name))
                        .map(|marker| marker.get_width())
                        .unwrap_or(0.0),
                )
            })
            .unwrap_or((0.0, 0.0))
    }

    #[inline]
    pub fn get_bg_color(&self) -> Color {
        self.get_scene_style().get_fill_color().cloned().unwrap_or(Color::WHITE)
    }

    #[inline]
    pub fn get_named_gradspecs(&self) -> hash_map::Iter<String, Gradient> {
        self.named_gradspecs.iter()
    }

    #[inline]
    pub fn get_named_marker_ids(&self) -> hash_map::Iter<String, MarkerId> {
        self.named_markers.iter()
    }

    #[inline]
    pub fn get_named_markers(&self) -> NamedMarkersIter {
        NamedMarkersIter { theme: self, entries: self.named_markers.iter() }
    }

    #[inline]
    pub fn get_font(&self, style_id: Option<StyleId>) -> Option<&Font> {
        self.get_style(style_id).and_then(|s| s.get_font())
    }

    #[inline]
    pub fn get_serif_font(&self) -> Option<&Font> {
        self.default_fonts.get(&GenericFontFamily::Serif)
    }

    #[inline]
    pub fn get_sans_serif_font(&self) -> Option<&Font> {
        self.default_fonts.get(&GenericFontFamily::SansSerif)
    }

    #[inline]
    pub fn get_sans_cursive_font(&self) -> Option<&Font> {
        self.default_fonts.get(&GenericFontFamily::Cursive)
    }

    #[inline]
    pub fn get_sans_monospace_font(&self) -> Option<&Font> {
        self.default_fonts.get(&GenericFontFamily::Monospace)
    }

    pub fn simple_demo() -> Self {
        let gradient_v_stops = vec![Color::WHITE, Color::BLACK];
        let gradient_h_stops = vec![Color::rgba8(0, 0xff, 0, 64), Color::rgba8(0xff, 0, 0xff, 64)];
        let gradient_r_stops = vec![Color::WHITE, Color::rgb8(0xff, 0, 0)];
        let dark_gradient_r_stops = vec![Color::BLACK, Color::rgb8(0xff, 0, 0xff)];

        let linear_gradients = vec![
            ("gradient-v", UnitPoint::TOP, UnitPoint::BOTTOM, gradient_v_stops.as_slice()),
            ("gradient-h", UnitPoint::LEFT, UnitPoint::RIGHT, gradient_h_stops.as_slice()),
        ];

        let radial_gradients = vec![
            ("gradient-r", 1., gradient_r_stops.as_slice()),
            ("dark-gradient-r", 1., dark_gradient_r_stops.as_slice()),
        ];

        let strokes = vec![
            ("line-1", Stroke::new().with_brush(Color::rgb8(0, 0x80, 0x80)).with_width(3.)),
            ("line-2", Stroke::new().with_brush(Color::rgb8(0x80, 0x80, 0)).with_width(0.5)),
            ("rect-2", Stroke::new().with_brush(Color::BLACK).with_width(1.)),
            ("circ-1", Stroke::new().with_brush(Color::rgb8(0xff, 0, 0)).with_width(5.)),
        ];

        let fills = vec![
            ("rect-1", Fill::Linear("gradient-v".into())),
            ("rect-2", Fill::Linear("gradient-h".into())),
            ("circ-1", Fill::Radial("gradient-r".into())),
        ];

        let dark_strokes =
            vec![("circ-1", Stroke::new().with_brush(Color::rgb8(0xa0, 0, 0xff)).with_width(5.))];

        let dark_fills = vec![
            (SCENE_NAME, Fill::Color(Color::BLACK)),
            ("circ-1", Fill::Radial("dark-gradient-r".into())),
        ];

        let variations =
            vec![("dark", Variation::new().with_strokes(dark_strokes).with_fills(dark_fills))];

        let styles = vec![
            ("border", Style::new().with_stroke(Stroke::new())),
            ("line-1", Style::new().with_named_stroke("line-1")),
            ("line-2", Style::new().with_named_stroke("line-2")),
            ("rect-1", Style::new().with_named_fill("rect-1")),
            ("rect-2", Style::new().with_named_fill("rect-2").with_named_stroke("rect-2")),
            ("circ-1", Style::new().with_named_fill("circ-1").with_named_stroke("circ-1")),
        ];

        Theme::new()
            .with_gradients(linear_gradients, radial_gradients)
            .with_strokes(strokes)
            .with_fills(fills)
            .with_variations(variations)
            .with_styles(styles)
    }
}

pub struct NamedMarkersIter<'a> {
    theme:   &'a Theme,
    entries: hash_map::Iter<'a, String, MarkerId>,
}

impl<'a> Iterator for NamedMarkersIter<'a> {
    type Item = Result<(&'a str, &'a Marker), VisError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((name, marker_id)) = self.entries.next() {
            if let Some(marker) = self.theme.get_marker(Some(*marker_id)) {
                Some(Ok((name, marker)))
            } else {
                Some(Err(VisError::marker_missing_for_id(*marker_id)))
            }
        } else {
            None
        }
    }
}
