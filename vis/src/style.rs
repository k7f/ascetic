use kurbo::{Point, Rect};
use crate::{Crumb, Variation, Tweener, Font};

#[derive(Clone, Copy, Debug)]
pub struct StyleId(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Rgba32(u32),
}

impl Color {
    pub const fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Color::Rgba32(((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | 0xff)
    }

    pub const fn rgb32(r: u32, g: u32, b: u32) -> Self {
        Color::Rgba32((r << 24) | (g << 16) | (b << 8) | 0xff)
    }

    pub const fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color::Rgba32(((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32))
    }

    pub const fn rgba32(r: u32, g: u32, b: u32, a: u32) -> Self {
        Color::Rgba32((r << 24) | (g << 16) | (b << 8) | a)
    }

    pub fn rgb(r: f64, g: f64, b: f64) -> Self {
        let r = (r.max(0.0).min(1.0) * 255.0).round() as u32;
        let g = (g.max(0.0).min(1.0) * 255.0).round() as u32;
        let b = (b.max(0.0).min(1.0) * 255.0).round() as u32;
        Color::Rgba32((r << 24) | (g << 16) | (b << 8) | 0xff)
    }

    pub fn rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        let r = (r.max(0.0).min(1.0) * 255.0).round() as u32;
        let g = (g.max(0.0).min(1.0) * 255.0).round() as u32;
        let b = (b.max(0.0).min(1.0) * 255.0).round() as u32;
        let a = (a.max(0.0).min(1.0) * 255.0).round() as u32;
        Color::Rgba32((r << 24) | (g << 16) | (b << 8) | a)
    }

    pub fn with_alpha(self, a: f64) -> Self {
        let a = (a.max(0.0).min(1.0) * 255.0).round() as u32;
        Color::Rgba32((self.as_u32() & !0xff) | a)
    }

    pub fn from_hsva(hue: f64, saturation: f64, value: f64, alpha: f64) -> Self {
        let hue = if hue < 0.0 {
            hue + 6.0
        } else if hue >= 6.0 {
            hue - 6.0
        } else {
            hue
        };
        let chroma = value * saturation;
        let hue_trunc = hue.trunc();
        let fract = (hue - hue_trunc) * chroma;
        let bottom = value - chroma;
        let red;
        let green;
        let blue;

        if hue_trunc < 1.0 {
            red = value;
            blue = bottom;
            green = bottom + fract;
        } else if hue_trunc < 2.0 {
            green = value;
            blue = bottom;
            red = value - fract;
        } else if hue_trunc < 3.0 {
            green = value;
            red = bottom;
            blue = bottom + fract;
        } else if hue_trunc < 4.0 {
            blue = value;
            red = bottom;
            green = value - fract;
        } else if hue_trunc < 5.0 {
            blue = value;
            green = bottom;
            red = bottom + fract;
        } else {
            red = value;
            green = bottom;
            blue = value - fract;
        }

        Color::rgba8(
            red.trunc() as u8,
            green.trunc() as u8,
            blue.trunc() as u8,
            alpha.trunc() as u8,
        )
    }

    #[inline]
    pub fn as_u32(self) -> u32 {
        match self {
            Color::Rgba32(rgba) => rgba,
        }
    }

    /// Returns four components, red, green, blue, and alpha, all in
    /// the range [0..255].
    #[inline]
    pub fn as_rgba8(self) -> (u8, u8, u8, u8) {
        match self {
            Color::Rgba32(rgba) => (
                (rgba >> 24 & 255) as u8,
                ((rgba >> 16) & 255) as u8,
                ((rgba >> 8) & 255) as u8,
                (rgba & 255) as u8,
            ),
        }
    }

    /// _Hue_ should be `None` iff _saturation_ is zero (for any
    /// _value_ of gray).
    pub fn as_hsva(self) -> (Option<f64>, f64, f64, f64) {
        let Rgba::<f64>(red, green, blue, alpha) = self.into();
        let gb = green - blue;
        let br = blue - red;
        let rg = red - green;

        if gb > 0.0 {
            if rg > 0.0 {
                // r > g > b
                // chroma = red - blue
                (Some(gb / -br), -br / red, red, alpha)
            } else if br > 0.0 {
                // g > b > r
                // chroma = green - red
                (Some(br / -rg + 2.0), -rg / green, green, alpha)
            } else {
                // r <= g > b <= r
                // chroma = green - blue
                (Some(br / gb + 2.0), gb / green, green, alpha)
            }
        } else if br > 0.0 {
            if rg > 0.0 {
                // b > r > g
                // chroma = blue - green
                (Some(rg / -gb + 4.0), -gb / blue, blue, alpha)
            } else {
                // g <= b > r <= g
                // chroma = blue - red
                (Some(rg / br + 4.0), br / blue, blue, alpha)
            }
        } else if rg > 0.0 {
            // b <= r > g <= b
            // chroma = red - green
            (Some(gb / rg + 6.0), rg / red, red, alpha)
        } else {
            // r = g = b
            (None, 0.0, red, alpha)
        }
    }

    pub const BLACK: Color = Color::rgb8(0, 0, 0);
    pub const WHITE: Color = Color::rgb8(255, 255, 255);
}

pub struct Rgba<T>(pub T, pub T, pub T, pub T);

impl<T: From<u32>> From<Color> for Rgba<T> {
    #[inline]
    fn from(color: Color) -> Self {
        match color {
            Color::Rgba32(rgba) => Rgba(
                ((rgba >> 24) & 255).into(),
                ((rgba >> 16) & 255).into(),
                ((rgba >> 8) & 255).into(),
                (rgba & 255).into(),
            ),
        }
    }
}

impl std::fmt::Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "#{:08x}", self.as_u32())
    }
}

#[derive(Clone, Debug)]
pub struct Stroke {
    brush: Color,
    width: f64,
}

impl Default for Stroke {
    fn default() -> Self {
        Stroke { brush: Color::BLACK, width: 1.0 }
    }
}

impl Stroke {
    pub const fn new() -> Self {
        Stroke { brush: Color::BLACK, width: 1.0 }
    }

    pub fn with_brush(mut self, brush: Color) -> Self {
        self.set_brush(brush);
        self
    }

    pub fn with_width(mut self, width: f64) -> Self {
        self.set_width(width);
        self
    }

    #[inline]
    pub fn set_brush(&mut self, brush: Color) {
        self.brush = brush;
    }

    #[inline]
    pub fn set_width(&mut self, width: f64) {
        self.width = width;
    }

    #[inline]
    pub fn get_brush(&self) -> &Color {
        &self.brush
    }

    #[inline]
    pub fn get_mut_brush(&mut self) -> &mut Color {
        &mut self.brush
    }

    #[inline]
    pub fn get_width(&self) -> f64 {
        self.width
    }
}

#[derive(Clone, Debug)]
pub enum Fill {
    Color(Color),
    Linear(String),
    Radial(String),
}

impl Default for Fill {
    fn default() -> Self {
        Fill::Color(Color::WHITE)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UnitPoint {
    u: f64,
    v: f64,
}

impl UnitPoint {
    pub const TOP_LEFT: UnitPoint = UnitPoint::new(0.0, 0.0);
    pub const TOP: UnitPoint = UnitPoint::new(0.5, 0.0);
    pub const TOP_RIGHT: UnitPoint = UnitPoint::new(1.0, 0.0);
    pub const LEFT: UnitPoint = UnitPoint::new(0.0, 0.5);
    pub const CENTER: UnitPoint = UnitPoint::new(0.5, 0.5);
    pub const RIGHT: UnitPoint = UnitPoint::new(1.0, 0.5);
    pub const BOTTOM_LEFT: UnitPoint = UnitPoint::new(0.0, 1.0);
    pub const BOTTOM: UnitPoint = UnitPoint::new(0.5, 1.0);
    pub const BOTTOM_RIGHT: UnitPoint = UnitPoint::new(1.0, 1.0);

    pub const fn new(u: f64, v: f64) -> UnitPoint {
        UnitPoint { u, v }
    }

    pub fn resolve(self, rect: Rect) -> Point {
        Point::new(rect.x0 + self.u * (rect.x1 - rect.x0), rect.y0 + self.v * (rect.y1 - rect.y0))
    }
}

#[derive(Debug, Clone)]
pub enum ScaleMode {
    Fit,
    Fill,
}

#[derive(Debug, Clone)]
pub struct GradientStop {
    pub pos:   f32,
    pub color: Color,
}

impl PartialEq for GradientStop {
    fn eq(&self, other: &GradientStop) -> bool {
        self.color == other.color && self.pos.to_bits() == other.pos.to_bits()
    }
}

impl Eq for GradientStop {}

impl std::hash::Hash for GradientStop {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.color.hash(state);
        self.pos.to_bits().hash(state);
    }
}

pub trait GradientStops {
    fn into_vec(self) -> Vec<GradientStop>;
}

impl GradientStops for Vec<GradientStop> {
    fn into_vec(self) -> Vec<GradientStop> {
        self
    }
}

impl<'a> GradientStops for &'a [GradientStop] {
    fn into_vec(self) -> Vec<GradientStop> {
        self.to_owned()
    }
}

impl<'a> GradientStops for &'a [Color] {
    fn into_vec(self) -> Vec<GradientStop> {
        if self.is_empty() {
            Vec::new()
        } else {
            let denom = (self.len() - 1).max(1) as f32;
            self.iter()
                .enumerate()
                .map(|(i, c)| GradientStop { pos: (i as f32) / denom, color: c.to_owned() })
                .collect()
        }
    }
}

#[derive(Clone, Debug)]
pub enum Gradient {
    Linear(UnitPoint, UnitPoint, Vec<GradientStop>),
    Radial(f64, Vec<GradientStop>),
}

#[derive(Clone, Copy, Debug)]
pub struct MarkerId(pub usize);

#[derive(Clone, Debug)]
pub struct Marker {
    width:      f64,
    height:     f64,
    refx:       f64,
    refy:       f64,
    orient:     Option<f64>,
    crumb:      Crumb,
    style_name: Option<String>,
    // FIXME resolve style name to StyleId: either introduce builder
    // pattern for Theme, or add update to all relevant `with_` methods
}

impl Marker {
    pub fn new(crumb: Crumb) -> Self {
        Marker {
            width: 0.0,
            height: 0.0,
            refx: 0.0,
            refy: 0.0,
            orient: None,
            crumb,
            style_name: None,
        }
    }

    pub fn with_size(mut self, width: f64, height: f64) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_refxy(mut self, refx: f64, refy: f64) -> Self {
        self.refx = refx;
        self.refy = refy;
        self
    }

    pub fn with_named_style<S: AsRef<str>>(mut self, name: S) -> Self {
        self.style_name = Some(name.as_ref().into());
        self
    }

    #[inline]
    pub fn get_width(&self) -> f64 {
        self.width
    }

    #[inline]
    pub fn get_height(&self) -> f64 {
        self.height
    }

    #[inline]
    pub fn get_refx(&self) -> f64 {
        self.refx
    }

    #[inline]
    pub fn get_refy(&self) -> f64 {
        self.refy
    }

    #[inline]
    pub fn get_orient(&self) -> Option<f64> {
        self.orient
    }

    #[inline]
    pub fn get_crumb(&self) -> &Crumb {
        &self.crumb
    }

    #[inline]
    pub fn get_style_name(&self) -> Option<&str> {
        self.style_name.as_deref()
    }
}

#[derive(Clone, Default, Debug)]
pub(crate) struct MarkerSuit {
    start_name: Option<String>,
    mid_name:   Option<String>,
    end_name:   Option<String>,
    // FIXME resolve marker names to ids: either introduce builder
    // pattern for Theme, or add update to all relevant `with_` methods
}

impl MarkerSuit {
    pub(crate) const fn new() -> Self {
        MarkerSuit { start_name: None, mid_name: None, end_name: None }
    }

    pub(crate) fn get_start_name(&self) -> Option<&str> {
        self.start_name.as_deref()
    }

    pub(crate) fn get_mid_name(&self) -> Option<&str> {
        self.mid_name.as_deref()
    }

    pub(crate) fn get_end_name(&self) -> Option<&str> {
        self.end_name.as_deref()
    }
}

#[derive(Clone, Default, Debug)]
pub struct Style {
    stroke_name:    Option<String>,
    fill_name:      Option<String>,
    stroke:         Option<Stroke>,
    fill:           Option<Fill>,
    stroke_tweener: Option<Tweener<Stroke>>,
    fill_tweener:   Option<Tweener<Fill>>,
    markers:        MarkerSuit,
    font:           Option<Font>,
}

impl Style {
    pub const fn new() -> Self {
        Style {
            stroke_name:    None,
            fill_name:      None,
            stroke:         None,
            fill:           None,
            stroke_tweener: None,
            fill_tweener:   None,
            markers:        MarkerSuit::new(),
            font:           None,
        }
    }

    pub fn with_stroke(mut self, stroke: Stroke) -> Self {
        self.set_stroke(stroke);
        self
    }

    pub fn with_named_stroke<S: AsRef<str>>(mut self, name: S) -> Self {
        self.stroke_name = Some(name.as_ref().into());
        self
    }

    pub fn with_fill(mut self, fill: Fill) -> Self {
        self.set_fill(fill);
        self
    }

    pub fn with_named_fill<S: AsRef<str>>(mut self, name: S) -> Self {
        self.fill_name = Some(name.as_ref().into());
        self
    }

    pub fn with_named_start_marker<S: AsRef<str>>(mut self, name: S) -> Self {
        self.markers.start_name = Some(name.as_ref().into());
        self
    }

    pub fn with_named_mid_marker<S: AsRef<str>>(mut self, name: S) -> Self {
        self.markers.mid_name = Some(name.as_ref().into());
        self
    }

    pub fn with_named_end_marker<S: AsRef<str>>(mut self, name: S) -> Self {
        self.markers.end_name = Some(name.as_ref().into());
        self
    }

    pub fn with_font(mut self, font: Font) -> Self {
        self.set_font(font);
        self
    }

    pub fn resolve_initially(&mut self, variation: &Variation) {
        if let Some(stroke) =
            self.stroke_name.as_ref().and_then(|n| variation.get_stroke_by_name(n))
        {
            self.stroke = Some(stroke.clone());
        }

        if let Some(fill) = self.fill_name.as_ref().and_then(|n| variation.get_fill_by_name(n)) {
            self.fill = Some(fill.clone());
        }
    }

    pub fn resolve<V, I>(&mut self, variation: &Variation, path: I)
    where
        V: AsRef<str>,
        I: IntoIterator<Item = V> + Clone,
    {
        if let Some(stroke) =
            self.stroke_name.as_ref().and_then(|n| variation.get_stroke_by_path(path.clone(), n))
        {
            self.stroke = Some(stroke.clone());
        }

        if let Some(fill) =
            self.fill_name.as_ref().and_then(|n| variation.get_fill_by_path(path.clone(), n))
        {
            self.fill = Some(fill.clone());
        }
    }

    pub fn start_original_resolution(&mut self, variation: &Variation, max_subdivision: usize) {
        self.stroke_tweener = None;
        self.fill_tweener = None;

        if let Some(stroke_to) =
            self.stroke_name.as_ref().and_then(|n| variation.get_stroke_by_name(n))
        {
            if let Some(ref stroke_from) = self.stroke {
                self.stroke_tweener =
                    Some(Tweener::new(stroke_from.clone(), stroke_to.clone(), max_subdivision));
            }
        }

        if let Some(fill_to) = self.fill_name.as_ref().and_then(|n| variation.get_fill_by_name(n)) {
            if let Some(ref fill_from) = self.fill {
                self.fill_tweener =
                    Some(Tweener::new(fill_from.clone(), fill_to.clone(), max_subdivision));
            }
        }
    }

    pub fn start_resolution<V, I>(&mut self, variation: &Variation, path: I, max_subdivision: usize)
    where
        V: AsRef<str>,
        I: IntoIterator<Item = V> + Clone,
    {
        self.stroke_tweener = None;
        self.fill_tweener = None;

        if let Some(stroke_to) =
            self.stroke_name.as_ref().and_then(|n| variation.get_stroke_by_path(path.clone(), n))
        {
            if let Some(ref stroke_from) = self.stroke {
                self.stroke_tweener =
                    Some(Tweener::new(stroke_from.clone(), stroke_to.clone(), max_subdivision));
            }
        }

        if let Some(fill_to) =
            self.fill_name.as_ref().and_then(|n| variation.get_fill_by_path(path.clone(), n))
        {
            if let Some(ref fill_from) = self.fill {
                self.fill_tweener =
                    Some(Tweener::new(fill_from.clone(), fill_to.clone(), max_subdivision));
            }
        }
    }

    pub fn step_resolution(&mut self, amount: f64) {
        if let Some(ref mut tweener) = self.stroke_tweener {
            if let Some(stroke) = tweener.tween_on(amount) {
                match &mut self.stroke {
                    Some(dest) => dest.clone_from(stroke),
                    dest => *dest = Some(stroke.clone()),
                }
            }
        }

        if let Some(ref mut tweener) = self.fill_tweener {
            if let Some(fill) = tweener.tween_on(amount) {
                match &mut self.fill {
                    Some(dest) => dest.clone_from(fill),
                    dest => *dest = Some(fill.clone()),
                }
            }
        }
    }

    #[inline]
    pub fn set_stroke(&mut self, stroke: Stroke) {
        self.stroke = Some(stroke);
    }

    #[inline]
    pub fn clear_stroke(&mut self) {
        self.stroke = None;
    }

    #[inline]
    pub fn set_fill(&mut self, fill: Fill) {
        self.fill = Some(fill);
    }

    #[inline]
    pub fn clear_fill(&mut self) {
        self.fill = None;
    }

    #[inline]
    pub fn get_stroke(&self) -> Option<&Stroke> {
        self.stroke.as_ref()
    }

    #[inline]
    pub fn get_mut_stroke(&mut self) -> Option<&mut Stroke> {
        self.stroke.as_mut()
    }

    #[inline]
    pub fn get_fill(&self) -> Option<&Fill> {
        self.fill.as_ref()
    }

    #[inline]
    pub fn get_mut_fill(&mut self) -> Option<&mut Fill> {
        self.fill.as_mut()
    }

    #[inline]
    pub fn get_fill_color(&self) -> Option<&Color> {
        match self.fill {
            Some(Fill::Color(ref c)) => Some(c),
            _ => None,
        }
    }

    #[inline]
    pub fn get_fill_gradient_name(&self) -> Option<&str> {
        match self.fill {
            Some(Fill::Linear(ref name)) | Some(Fill::Radial(ref name)) => Some(name.as_str()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn get_markers(&self) -> &MarkerSuit {
        &self.markers
    }

    #[inline]
    pub(crate) fn set_font(&mut self, font: Font) {
        self.font = Some(font);
    }

    #[inline]
    pub(crate) fn get_font(&self) -> Option<&Font> {
        self.font.as_ref()
    }
}
