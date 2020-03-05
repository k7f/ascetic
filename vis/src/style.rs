use std::fmt::{self, Write};
use piet_common::{Color, UnitPoint, GradientStop, kurbo::Rect};

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

    pub fn write_svg<W: Write>(&self, svg: &mut W) -> fmt::Result {
        let rgba = self.brush.as_rgba_u32();

        // FIXME opacity
        write!(
            svg,
            "stroke=\"#{:06x}\" stroke-opacity=\"{}\" stroke-width=\"{}\"",
            rgba >> 8,
            (rgba & 0x0ff) as f64 / 255.,
            self.width
        )
    }
}

impl Default for Stroke {
    fn default() -> Self {
        Stroke { brush: Color::BLACK, width: 1.0 }
    }
}

#[derive(Clone, Debug)]
pub enum GradSpec {
    Linear(UnitPoint, UnitPoint, Vec<GradientStop>),
}

impl GradSpec {
    pub fn write_svg_with_name<W: Write, S: AsRef<str>>(
        &self,
        svg: &mut W,
        name: S,
    ) -> fmt::Result {
        write!(svg, "    <linearGradient id=\"{}\"", name.as_ref())?;

        match self {
            GradSpec::Linear(start, end, stops) => {
                let start = start.resolve(Rect::new(0., 0., 100., 100.));
                let end = end.resolve(Rect::new(0., 0., 100., 100.));

                writeln!(
                    svg,
                    " x1=\"{}%\" y1=\"{}%\" x2=\"{}%\" y2=\"{}%\">",
                    start.x, start.y, end.x, end.y
                )?;

                for stop in stops.iter() {
                    let rgba = stop.color.as_rgba_u32();

                    // FIXME opacity
                    writeln!(
                        svg,
                        "      <stop offset=\"{}\" stop-color=\"#{:06x}\" stop-opacity=\"{:.2}\"/>",
                        stop.pos,
                        rgba >> 8,
                        (rgba & 0x0ff) as f64 / 255.
                    )?;
                }
            }
        }

        writeln!(svg, "    </linearGradient>")
    }
}

#[derive(Clone, Debug)]
pub enum Fill {
    Color(Color),
    Linear(String),
}

impl Fill {
    pub fn write_svg<W: Write>(&self, svg: &mut W) -> fmt::Result {
        match self {
            Fill::Color(color) => {
                let rgba = color.as_rgba_u32();

                // FIXME opacity
                write!(
                    svg,
                    "fill=\"#{:06x}\" fill-opacity=\"{}\"",
                    rgba >> 8,
                    (rgba & 0x0ff) as f64 / 255.,
                )
            }
            Fill::Linear(ref name) => write!(svg, "fill=\"url(#{})\"", name),
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
            Some(Fill::Linear(ref name)) => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn write_svg<W: Write>(&self, svg: &mut W) -> fmt::Result {
        if let Some(ref stroke) = self.stroke {
            stroke.write_svg(svg)?;
            write!(svg, " ")?;
        }

        if let Some(ref fill) = self.fill {
            fill.write_svg(svg)
        } else {
            write!(svg, "fill=\"none\"")
        }
    }
}
