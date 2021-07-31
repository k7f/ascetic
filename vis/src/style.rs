use piet::{Color, UnitPoint, GradientStop};
use crate::{Crumb, Variation, Tweener};

#[derive(Clone, Copy, Debug)]
pub struct StyleId(pub usize);

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

#[derive(Clone, Debug)]
pub enum GradSpec {
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
}
