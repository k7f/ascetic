use std::io;
use piet_common::{Color, UnitPoint, GradientStop, kurbo::Rect};
use crate::{WriteSvg, WriteSvgWithName};

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
    pub fn get_width(&self) -> f64 {
        self.width
    }
}

impl Default for Stroke {
    fn default() -> Self {
        Stroke { brush: Color::BLACK, width: 1.0 }
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

impl WriteSvg for Fill {
    fn write_svg<W: io::Write>(&self, mut svg: W) -> io::Result<()> {
        match self {
            Fill::Color(ref color) => color.write_svg_with_name(&mut svg, "fill"),
            Fill::Linear(ref name) => write!(svg, "fill=\"url(#{})\"", name),
            Fill::Radial(ref name) => write!(svg, "fill=\"url(#{})\"", name),
        }
    }
}

impl Default for Fill {
    fn default() -> Self {
        Fill::Color(Color::WHITE)
    }
}

#[derive(Clone, Default, Debug)]
pub struct Style {
    stroke: Option<Stroke>,
    fill:   Option<Fill>,
}

impl Style {
    pub const fn new() -> Self {
        Style { stroke: None, fill: None }
    }

    pub fn with_stroke(mut self, stroke: Stroke) -> Self {
        self.set_stroke(stroke);
        self
    }

    pub fn with_fill(mut self, fill: Fill) -> Self {
        self.set_fill(fill);
        self
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
    pub fn get_fill(&self) -> Option<&Fill> {
        self.fill.as_ref()
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
