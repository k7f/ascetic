use std::io::Write;
use kurbo::{Shape, Line, Rect, RoundedRect, Circle, Arc, BezPath, TranslateScale, Size};
use piet::Color;
use crate::{
    Scene, Theme, Style, StyleId, Stroke, Fill, GradSpec, Marker, style::MarkerSuit, Crumb,
    CrumbItem,
};

pub trait ToSvg {
    fn to_svg<S, M>(
        &self,
        theme: &Theme,
        out_size: S,
        out_margin: M,
    ) -> Result<String, Box<dyn std::error::Error>>
    where
        S: Into<Size>,
        M: Into<Size>;
}

impl ToSvg for Scene {
    fn to_svg<S, M>(
        &self,
        theme: &Theme,
        out_size: S,
        out_margin: M,
    ) -> Result<String, Box<dyn std::error::Error>>
    where
        S: Into<Size>,
        M: Into<Size>,
    {
        let out_size = out_size.into();
        let out_margin = out_margin.into();
        let out_scale = ((out_size.width - 2. * out_margin.width) / self.get_size().width)
            .min((out_size.height - 2. * out_margin.height) / self.get_size().height);
        let root_ts =
            TranslateScale::translate(out_margin.to_vec2()) * TranslateScale::scale(out_scale);

        let mut svg = Vec::new();

        writeln!(&mut svg, "<svg version=\"1.1\" baseProfile=\"full\"")?;
        writeln!(&mut svg, "     xmlns=\"http://www.w3.org/2000/svg\"")?;
        writeln!(&mut svg, "     xmlns:xlink=\"http://www.w3.org/1999/xlink\"")?;
        writeln!(&mut svg, "     xmlns:ev=\"http://www.w3.org/2001/xml-events\"")?;
        writeln!(
            &mut svg,
            "     width=\"{}\" height=\"{}\">",
            out_size.width.round(),
            out_size.height.round()
        )?;

        writeln!(&mut svg, "  <defs>")?;

        for (name, spec) in theme.get_named_gradspecs() {
            spec.write_svg_with_name(&mut svg, name)?;
        }

        for (name, id) in theme.get_named_marker_ids() {
            if let Some(marker) = theme.get_marker(Some(*id)) {
                (marker, theme).write_svg_with_name(&mut svg, name)?;
            } else {
                // FIXME error
                panic!()
            }
        }

        writeln!(&mut svg, "  </defs>")?;

        let bg_color = theme.get_bg_color();
        write!(&mut svg, "  <rect width=\"100%\" height=\"100%\" ")?;
        bg_color.write_svg_with_name(&mut svg, "fill")?;
        writeln!(&mut svg, " />")?;

        for CrumbItem(crumb_id, ts, style_id) in self.all_crumbs(root_ts) {
            if let Some(crumb) = self.get_crumb(crumb_id) {
                crumb.write_svg_with_style(&mut svg, ts, style_id, theme)?;
            } else {
                // FIXME
                panic!()
            }
        }

        writeln!(&mut svg, "</svg>")?;

        let svg = String::from_utf8(svg)?;

        Ok(svg)
    }
}

pub trait WriteSvg {
    fn write_svg<W: std::io::Write>(&self, svg: W) -> std::io::Result<()>;
}

pub trait WriteSvgWithStyle: WriteSvg {
    #[inline]
    fn write_svg_opt<W: std::io::Write>(
        &self,
        svg: W,
        _style: Option<&Style>,
    ) -> std::io::Result<()> {
        self.write_svg(svg)
    }

    fn write_svg_with_style<W: std::io::Write>(
        &self,
        svg: W,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> std::io::Result<()>;
}

impl WriteSvg for Stroke {
    fn write_svg<W: std::io::Write>(&self, mut svg: W) -> std::io::Result<()> {
        self.get_brush().write_svg_with_name(&mut svg, "stroke")?;
        write!(svg, " stroke-width=\"{}\"", self.get_width())
    }
}

impl WriteSvg for Fill {
    fn write_svg<W: std::io::Write>(&self, mut svg: W) -> std::io::Result<()> {
        match self {
            Fill::Color(ref color) => color.write_svg_with_name(&mut svg, "fill"),
            Fill::Linear(ref name) => write!(svg, "fill=\"url(#{})\"", name),
            Fill::Radial(ref name) => write!(svg, "fill=\"url(#{})\"", name),
        }
    }
}

impl WriteSvg for MarkerSuit {
    fn write_svg<W: std::io::Write>(&self, mut svg: W) -> std::io::Result<()> {
        if let Some(name) = self.get_start_name() {
            write!(svg, " marker-start=\"url(#{})\"", name)?;
        }

        if let Some(name) = self.get_mid_name() {
            write!(svg, " marker-mid=\"url(#{})\"", name)?;
        }

        if let Some(name) = self.get_end_name() {
            write!(svg, " marker-end=\"url(#{})\"", name)?;
        }

        Ok(())
    }
}

impl WriteSvg for Style {
    fn write_svg<W: std::io::Write>(&self, mut svg: W) -> std::io::Result<()> {
        if let Some(stroke) = self.get_stroke() {
            stroke.write_svg(&mut svg)?;
            write!(svg, " ")?;
        }

        if let Some(fill) = self.get_fill() {
            fill.write_svg(&mut svg)
        } else {
            write!(svg, "fill=\"none\"")
        }
    }
}

impl WriteSvg for Crumb {
    #[inline]
    fn write_svg<W: std::io::Write>(&self, mut svg: W) -> std::io::Result<()> {
        match self {
            Crumb::Line(line) => line.write_svg(&mut svg),
            Crumb::Rect(rect) => rect.write_svg(&mut svg),
            Crumb::RoundedRect(rr) => rr.write_svg(&mut svg),
            Crumb::Circle(circ) => circ.write_svg(&mut svg),
            Crumb::Arc(arc) => arc.write_svg(&mut svg),
            Crumb::Path(path) => path.write_svg(&mut svg),
        }
    }
}

impl WriteSvgWithStyle for Crumb {
    #[inline]
    fn write_svg_opt<W: std::io::Write>(
        &self,
        mut svg: W,
        style: Option<&Style>,
    ) -> std::io::Result<()> {
        match self {
            Crumb::Line(line) => line.write_svg_opt(&mut svg, style),
            Crumb::Rect(rect) => rect.write_svg_opt(&mut svg, style),
            Crumb::RoundedRect(rr) => rr.write_svg_opt(&mut svg, style),
            Crumb::Circle(circ) => circ.write_svg_opt(&mut svg, style),
            Crumb::Arc(arc) => arc.write_svg_opt(&mut svg, style),
            Crumb::Path(path) => path.write_svg_opt(&mut svg, style),
        }
    }

    #[inline]
    fn write_svg_with_style<W: std::io::Write>(
        &self,
        mut svg: W,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> std::io::Result<()> {
        match self {
            Crumb::Line(line) => line.write_svg_with_style(&mut svg, ts, style_id, theme),
            Crumb::Rect(rect) => rect.write_svg_with_style(&mut svg, ts, style_id, theme),
            Crumb::RoundedRect(rr) => rr.write_svg_with_style(&mut svg, ts, style_id, theme),
            Crumb::Circle(circ) => circ.write_svg_with_style(&mut svg, ts, style_id, theme),
            Crumb::Arc(arc) => arc.write_svg_with_style(&mut svg, ts, style_id, theme),
            Crumb::Path(path) => path.write_svg_with_style(&mut svg, ts, style_id, theme),
        }
    }
}

impl WriteSvg for Line {
    fn write_svg<W: std::io::Write>(&self, mut svg: W) -> std::io::Result<()> {
        writeln!(
            svg,
            "  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" />",
            self.p0.x, self.p0.y, self.p1.x, self.p1.y
        )
    }
}

impl WriteSvgWithStyle for Line {
    fn write_svg_opt<W: std::io::Write>(
        &self,
        mut svg: W,
        style: Option<&Style>,
    ) -> std::io::Result<()> {
        if let Some(style) = style {
            write!(
                svg,
                "  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" />",
                self.p0.x, self.p0.y, self.p1.x, self.p1.y
            )?;
            if let Some(stroke) = style.get_stroke() {
                stroke.write_svg(&mut svg)?;
            }
            writeln!(svg, "/>")
        } else {
            self.write_svg(svg)
        }
    }

    fn write_svg_with_style<W: std::io::Write>(
        &self,
        mut svg: W,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> std::io::Result<()> {
        let p0 = ts * self.p0;
        let p1 = ts * self.p1;
        let style = theme.get_style(style_id).unwrap_or_else(|| theme.get_default_style());

        write!(svg, "  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" ", p0.x, p0.y, p1.x, p1.y)?;

        if let Some(stroke) = style.get_stroke() {
            stroke.write_svg(&mut svg)?;
        }

        style.get_markers().write_svg(&mut svg)?;

        writeln!(svg, " />")
    }
}

impl WriteSvg for Rect {
    fn write_svg<W: std::io::Write>(&self, mut svg: W) -> std::io::Result<()> {
        writeln!(
            svg,
            "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" />",
            self.x0,
            self.y0,
            self.width(),
            self.height(),
        )
    }
}

impl WriteSvgWithStyle for Rect {
    fn write_svg_opt<W: std::io::Write>(
        &self,
        mut svg: W,
        style: Option<&Style>,
    ) -> std::io::Result<()> {
        if let Some(style) = style {
            write!(
                svg,
                "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" />",
                self.x0,
                self.y0,
                self.width(),
                self.height(),
            )?;
            style.write_svg(&mut svg)?;
            writeln!(svg, "/>")
        } else {
            self.write_svg(svg)
        }
    }

    fn write_svg_with_style<W: std::io::Write>(
        &self,
        mut svg: W,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> std::io::Result<()> {
        let rect = ts * *self;

        write!(
            svg,
            "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" ",
            rect.x0,
            rect.y0,
            rect.width(),
            rect.height()
        )?;

        if let Some(style) = theme.get_style(style_id) {
            style.write_svg(&mut svg)?;
        }

        writeln!(svg, "/>")
    }
}

impl WriteSvg for RoundedRect {
    fn write_svg<W: std::io::Write>(&self, mut svg: W) -> std::io::Result<()> {
        let rect = &self.rect();
        if let Some(radius) = self.radii().as_single_radius() {
            writeln!(
                svg,
                "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" />",
                rect.x0,
                rect.y0,
                rect.width(),
                rect.height(),
                radius,
            )
        } else {
            writeln!(
                svg,
                "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" />",
                rect.x0,
                rect.y0,
                rect.width(),
                rect.height(),
            )
        }
    }
}

impl WriteSvgWithStyle for RoundedRect {
    fn write_svg_opt<W: std::io::Write>(
        &self,
        mut svg: W,
        style: Option<&Style>,
    ) -> std::io::Result<()> {
        if let Some(style) = style {
            let rect = &self.rect();
            if let Some(radius) = self.radii().as_single_radius() {
                write!(
                    svg,
                    "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" />",
                    rect.x0,
                    rect.y0,
                    rect.width(),
                    rect.height(),
                    radius,
                )?;
            } else {
                write!(
                    svg,
                    "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" />",
                    rect.x0,
                    rect.y0,
                    rect.width(),
                    rect.height(),
                )?;
            }
            style.write_svg(&mut svg)?;
            writeln!(svg, "/>")
        } else {
            self.write_svg(svg)
        }
    }

    fn write_svg_with_style<W: std::io::Write>(
        &self,
        mut svg: W,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> std::io::Result<()> {
        let rr = ts * *self;
        let rect = &rr.rect();
        if let Some(radius) = rr.radii().as_single_radius() {
            write!(
                svg,
                "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" ",
                rect.x0,
                rect.y0,
                rect.width(),
                rect.height(),
                radius,
            )?;
        } else {
            write!(
                svg,
                "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" ",
                rect.x0,
                rect.y0,
                rect.width(),
                rect.height(),
            )?;
        }

        if let Some(style) = theme.get_style(style_id) {
            style.write_svg(&mut svg)?;
        }

        writeln!(svg, "/>")
    }
}

impl WriteSvg for Circle {
    fn write_svg<W: std::io::Write>(&self, mut svg: W) -> std::io::Result<()> {
        writeln!(
            svg,
            "  <circle cx=\"{}\" cy=\"{}\" r=\"{}\" />",
            self.center.x, self.center.y, self.radius
        )
    }
}

impl WriteSvgWithStyle for Circle {
    fn write_svg_opt<W: std::io::Write>(
        &self,
        mut svg: W,
        style: Option<&Style>,
    ) -> std::io::Result<()> {
        if let Some(style) = style {
            write!(
                svg,
                "  <circle cx=\"{}\" cy=\"{}\" r=\"{}\" ",
                self.center.x, self.center.y, self.radius
            )?;
            style.write_svg(&mut svg)?;
            writeln!(svg, "/>")
        } else {
            self.write_svg(svg)
        }
    }

    fn write_svg_with_style<W: std::io::Write>(
        &self,
        mut svg: W,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> std::io::Result<()> {
        let center = ts * self.center;
        let radius = ts.as_tuple().1 * self.radius;

        write!(svg, "  <circle cx=\"{}\" cy=\"{}\" r=\"{}\" ", center.x, center.y, radius)?;

        if let Some(style) = theme.get_style(style_id) {
            style.write_svg(&mut svg)?;
        }

        writeln!(svg, "/>")
    }
}

impl WriteSvg for Arc {
    fn write_svg<W: std::io::Write>(&self, svg: W) -> std::io::Result<()> {
        // FIXME use `A` path element
        let path = BezPath::from_vec(self.path_elements(0.1).collect());

        path.write_svg(svg)
    }
}

impl WriteSvgWithStyle for Arc {
    fn write_svg_opt<W: std::io::Write>(
        &self,
        svg: W,
        style: Option<&Style>,
    ) -> std::io::Result<()> {
        // FIXME use `A` path element
        let path = BezPath::from_vec(self.path_elements(0.1).collect());

        path.write_svg_opt(svg, style)
    }

    fn write_svg_with_style<W: std::io::Write>(
        &self,
        svg: W,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> std::io::Result<()> {
        // FIXME use `A` path element
        let path = BezPath::from_vec(self.path_elements(0.1).collect());

        path.write_svg_with_style(svg, ts, style_id, theme)
    }
}

impl WriteSvg for BezPath {
    fn write_svg<W: std::io::Write>(&self, mut svg: W) -> std::io::Result<()> {
        write!(svg, "  <path d=\"")?;
        self.write_to(&mut svg)?;
        writeln!(svg, "\" />")
    }
}

impl WriteSvgWithStyle for BezPath {
    fn write_svg_opt<W: std::io::Write>(
        &self,
        mut svg: W,
        style: Option<&Style>,
    ) -> std::io::Result<()> {
        if let Some(style) = style {
            write!(svg, "  <path d=\"")?;
            self.write_to(&mut svg)?;
            write!(svg, "\" ")?;
            style.write_svg(&mut svg)?;
            writeln!(svg, "/>")
        } else {
            self.write_svg(svg)
        }
    }

    fn write_svg_with_style<W: std::io::Write>(
        &self,
        mut svg: W,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> std::io::Result<()> {
        write!(svg, "  <path d=\"")?;
        (ts * self.clone()).write_to(&mut svg)?;
        write!(svg, "\" ")?;

        if let Some(style) = theme.get_style(style_id) {
            style.write_svg(&mut svg)?;
        }

        writeln!(svg, "/>")
    }
}

pub trait WriteSvgWithName {
    fn write_svg_with_name<W: std::io::Write, S: AsRef<str>>(
        &self,
        svg: W,
        name: S,
    ) -> std::io::Result<()>;
}

impl WriteSvgWithName for Color {
    fn write_svg_with_name<W: std::io::Write, S: AsRef<str>>(
        &self,
        mut svg: W,
        name: S,
    ) -> std::io::Result<()> {
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

impl WriteSvgWithName for GradSpec {
    fn write_svg_with_name<W: std::io::Write, S: AsRef<str>>(
        &self,
        mut svg: W,
        name: S,
    ) -> std::io::Result<()> {
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

impl WriteSvgWithName for (&Marker, &Theme) {
    fn write_svg_with_name<W: std::io::Write, S: AsRef<str>>(
        &self,
        mut svg: W,
        name: S,
    ) -> std::io::Result<()> {
        write!(svg, "    <marker id=\"{}\" ", name.as_ref())?;
        if let Some(orient) = self.0.get_orient() {
            writeln!(svg, "orient=\"{}\"", orient)?;
        } else {
            writeln!(svg, "orient=\"auto\"")?;
        }
        writeln!(
            svg,
            "            markerWidth=\"{}\" markerHeight=\"{}\"",
            self.0.get_width(),
            self.0.get_height()
        )?;
        writeln!(
            svg,
            "            refX=\"{}\" refY=\"{}\">",
            self.0.get_refx(),
            self.0.get_refy()
        )?;

        self.0.get_crumb().write_svg_opt(
            &mut svg,
            self.0.get_style_name().and_then(|name| self.1.get_style_by_name(name)),
        )?;
        writeln!(svg, "    </marker>")
    }
}
