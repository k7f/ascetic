use std::io;
use piet::{Color, UnitPoint, GradientStop};
use kurbo::Rect;
use crate::{
    Variation, Tweener, WriteSvg, WriteSvgWithName, AsUsvgNodeWithName, AsUsvgStyle, AsUsvgStroke,
    AsUsvgFill, AsUsvgStop,
};

#[derive(Clone, Copy, Debug)]
pub struct StyleId(pub usize);

impl WriteSvgWithName for Color {
    fn write_svg_with_name<W: io::Write, S: AsRef<str>>(
        &self,
        mut svg: W,
        name: S,
    ) -> io::Result<()> {
        let rgba = self.as_rgba_u32();
        let name = name.as_ref();
        let stem = name.find('-').and_then(|pos| name.get(..pos)).unwrap_or(name);

        write!(
            svg,
            "{}=\"#{:06x}\" {}-opacity=\"{:.*}\"",
            name,
            rgba >> 8,
            stem,
            3, // FIXME precision
            (rgba & 0x0ff) as f64 / 255.,
        )
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

impl AsUsvgStroke for Stroke {
    fn as_usvg(&self) -> usvg::Stroke {
        let (red, green, blue, alpha) = self.brush.as_rgba8();

        if alpha == 0xff {
            usvg::Stroke {
                paint: usvg::Paint::Color(usvg::Color::new(red, green, blue)),
                width: self.width.into(),
                ..Default::default()
            }
        } else {
            usvg::Stroke {
                paint: usvg::Paint::Color(usvg::Color::new(red, green, blue)),
                opacity: (alpha as f64 / 255.0).into(),
                width: self.width.into(),
                ..Default::default()
            }
        }
    }
}

impl WriteSvg for Stroke {
    fn write_svg<W: io::Write>(&self, mut svg: W) -> io::Result<()> {
        self.brush.write_svg_with_name(&mut svg, "stroke")?;
        write!(svg, " stroke-width=\"{}\"", self.width)
    }
}

impl AsUsvgStop for GradientStop {
    fn as_usvg(&self) -> usvg::Stop {
        let (red, green, blue, alpha) = self.color.as_rgba8();

        usvg::Stop {
            offset:  usvg::StopOffset::new(self.pos.into()),
            color:   usvg::Color::new(red, green, blue),
            opacity: if alpha == 0xff { 1.0.into() } else { (alpha as f64 / 255.0).into() },
        }
    }
}

#[derive(Clone, Debug)]
pub enum GradSpec {
    Linear(UnitPoint, UnitPoint, Vec<GradientStop>),
    Radial(f64, Vec<GradientStop>),
}

impl WriteSvgWithName for GradSpec {
    fn write_svg_with_name<W: io::Write, S: AsRef<str>>(
        &self,
        mut svg: W,
        name: S,
    ) -> io::Result<()> {
        match self {
            GradSpec::Linear(start, end, stops) => {
                let start = start.resolve(Rect::new(0., 0., 100., 100.));
                let end = end.resolve(Rect::new(0., 0., 100., 100.));

                writeln!(
                    svg,
                    "    <linearGradient id=\"{}\" x1=\"{}%\" y1=\"{}%\" x2=\"{}%\" y2=\"{}%\">",
                    name.as_ref(),
                    start.x,
                    start.y,
                    end.x,
                    end.y
                )?;

                for stop in stops.iter() {
                    write!(svg, "      <stop offset=\"{}\" ", stop.pos)?;
                    stop.color.write_svg_with_name(&mut svg, "stop-color")?;
                    writeln!(svg, "/>")?;
                }

                writeln!(svg, "    </linearGradient>")
            }
            GradSpec::Radial(radius, stops) => {
                writeln!(
                    svg,
                    "    <radialGradient id=\"{}\" r=\"{}%\">",
                    name.as_ref(),
                    radius * 100.
                )?;

                for stop in stops.iter() {
                    write!(svg, "      <stop offset=\"{}\" ", stop.pos)?;
                    stop.color.write_svg_with_name(&mut svg, "stop-color")?;
                    writeln!(svg, "/>")?;
                }

                writeln!(svg, "    </radialGradient>")
            }
        }
    }
}

impl AsUsvgNodeWithName for GradSpec {
    fn as_usvg_node_with_name<S: AsRef<str>>(&self, name: S) -> usvg::NodeKind {
        match self {
            GradSpec::Linear(start, end, stops) => {
                let start = start.resolve(Rect::new(0., 0., 1., 1.));
                let end = end.resolve(Rect::new(0., 0., 1., 1.));

                usvg::NodeKind::LinearGradient(usvg::LinearGradient {
                    id:   name.as_ref().into(),
                    x1:   start.x,
                    y1:   start.y,
                    x2:   end.x,
                    y2:   end.y,
                    base: usvg::BaseGradient {
                        units:         usvg::Units::ObjectBoundingBox,
                        transform:     usvg::Transform::default(),
                        spread_method: usvg::SpreadMethod::Pad,
                        stops:         stops.iter().map(|stop| stop.as_usvg()).collect(),
                    },
                })
            }
            GradSpec::Radial(radius, stops) => {
                usvg::NodeKind::RadialGradient(usvg::RadialGradient {
                    id:   name.as_ref().into(),
                    r:    (*radius).into(),
                    cx:   0.5,
                    cy:   0.5,
                    fx:   0.5,
                    fy:   0.5,
                    base: usvg::BaseGradient {
                        units:         usvg::Units::ObjectBoundingBox,
                        transform:     usvg::Transform::default(),
                        spread_method: usvg::SpreadMethod::Pad,
                        stops:         stops.iter().map(|stop| stop.as_usvg()).collect(),
                    },
                })
            }
        }
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

impl AsUsvgFill for Fill {
    fn as_usvg(&self) -> usvg::Fill {
        let (paint, alpha) = match self {
            Fill::Color(color) => {
                let (red, green, blue, alpha) = color.as_rgba8();

                (usvg::Paint::Color(usvg::Color::new(red, green, blue)), alpha)
            }
            Fill::Linear(name) | Fill::Radial(name) => (usvg::Paint::Link(name.into()), 0xff),
        };

        if alpha == 0xff {
            usvg::Fill { paint, ..Default::default() }
        } else {
            usvg::Fill { paint, opacity: (alpha as f64 / 255.0).into(), ..Default::default() }
        }
    }
}

impl WriteSvg for Fill {
    fn write_svg<W: io::Write>(&self, mut svg: W) -> io::Result<()> {
        match self {
            Fill::Color(ref color) => color.write_svg_with_name(&mut svg, "fill"),
            Fill::Linear(ref name) => write!(svg, "fill=\"url(#{})\"", name),
            Fill::Radial(ref name) => write!(svg, "fill=\"url(#{})\"", name),
        }
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
            self.fill_name.as_ref().and_then(|n| variation.get_fill_by_path(path, n))
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
}

impl AsUsvgStyle for Style {
    #[inline]
    fn as_usvg(&self) -> (Option<usvg::Fill>, Option<usvg::Stroke>) {
        (self.fill.as_ref().map(|f| f.as_usvg()), self.stroke.as_ref().map(|s| s.as_usvg()))
    }
}

impl WriteSvg for Style {
    fn write_svg<W: io::Write>(&self, mut svg: W) -> io::Result<()> {
        if let Some(ref stroke) = self.stroke {
            stroke.write_svg(&mut svg)?;
            write!(svg, " ")?;
        }

        if let Some(ref fill) = self.fill {
            fill.write_svg(&mut svg)
        } else {
            write!(svg, "fill=\"none\"")
        }
    }
}
