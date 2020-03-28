use std::io;
use piet::{Color, UnitPoint, GradientStop};
use kurbo::Rect;
use crate::{Variation, Tweener, WriteSvg, WriteSvgWithName};

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

impl WriteSvg for Stroke {
    fn write_svg<W: io::Write>(&self, mut svg: W) -> io::Result<()> {
        self.brush.write_svg_with_name(&mut svg, "stroke")?;
        write!(svg, " stroke-width=\"{}\"", self.width)
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

    pub fn start_resolution<V, I>(&mut self, variation: &Variation, path: I)
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
                self.stroke_tweener = Some(Tweener::new(stroke_from.clone(), stroke_to.clone(), 1));
            }
        }

        if let Some(fill_to) =
            self.fill_name.as_ref().and_then(|n| variation.get_fill_by_path(path.clone(), n))
        {
            if let Some(ref fill_from) = self.fill {
                self.fill_tweener = Some(Tweener::new(fill_from.clone(), fill_to.clone(), 1));
            }
        }
    }

    pub fn step_resolution(&mut self, amount: f64) {
        if let Some(ref mut stroke_tweener) = self.stroke_tweener {
            let stroke = stroke_tweener.tween_on(amount);
            self.stroke = Some(stroke.clone());
        }

        if let Some(ref mut fill_tweener) = self.fill_tweener {
            let fill = fill_tweener.tween_on(amount);
            self.fill = Some(fill.clone());
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
