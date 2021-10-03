use std::io::Write;
use kurbo::{Shape, Line, Rect, RoundedRect, Circle, Arc, BezPath, TranslateScale, Size};
use crate::{
    Scene, Theme, Style, Color, Stroke, Fill, Gradient, Marker, style::MarkerSuit, Crumb, CrumbSet,
    TextLabel, PreprocessWithStyle,
};

pub trait ToSvg {
    fn to_svg<S, M>(
        &mut self,
        theme: &Theme,
        out_size: S,
        out_margin: M,
    ) -> Result<String, Box<dyn std::error::Error>>
    where
        S: Into<Size>,
        M: Into<Size>;
}

pub trait WriteSvg {
    fn write_svg<W: std::io::Write>(&self, svg: W, scale_factor: f64) -> std::io::Result<()>;
}

pub trait WriteSvgWithStyle {
    fn write_svg_with_style<W: std::io::Write>(
        &self,
        svg: W,
        ts: TranslateScale,
        style: Option<&Style>,
        theme: &Theme,
    ) -> std::io::Result<()>;
}

pub trait WriteSvgWithName {
    fn write_svg_with_name<W: std::io::Write, S: AsRef<str>>(
        &self,
        svg: W,
        name: S,
    ) -> std::io::Result<()>;
}

pub trait WriteSvgWithTheme {
    fn write_svg_with_theme<W: std::io::Write, S: AsRef<str>>(
        &self,
        svg: W,
        scale_factor: f64,
        name: S,
        theme: &Theme,
    ) -> std::io::Result<()>;
}

impl ToSvg for Scene {
    fn to_svg<S, M>(
        &mut self,
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
        let translation = out_margin.to_vec2();
        let scale_factor = ((out_size.width - 2. * out_margin.width) / self.get_size().width)
            .min((out_size.height - 2. * out_margin.height) / self.get_size().height);
        let root_ts = TranslateScale::translate(translation) * TranslateScale::scale(scale_factor);

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

        for entry in theme.get_named_markers() {
            let (name, marker) = entry?;

            marker.write_svg_with_theme(&mut svg, scale_factor, name, theme)?;
        }

        writeln!(&mut svg, "  </defs>")?;

        let bg_color = theme.get_bg_color();
        write!(&mut svg, "  <rect width=\"100%\" height=\"100%\" ")?;
        bg_color.write_svg_with_name(&mut svg, "fill")?;
        writeln!(&mut svg, " />")?;

        //let all_crumbs: Vec<_> = self.all_visible_crumbs(root_ts)?.collect();
        let visible_crumbs: CrumbSet = self.all_visible_crumbs(root_ts)?.collect();

        visible_crumbs.try_for_each_label(self, |label, ts, style_id| {
            let style = theme.get_style(style_id);

            label.preprocess_with_style(ts, style, theme)
        })?;

        for item in visible_crumbs.get_crumbs(self) {
            let (crumb, ts, style_id) = item?;
            let style = theme.get_style(style_id);

            crumb.write_svg_with_style(&mut svg, ts, style, theme)?;
        }

        writeln!(&mut svg, "</svg>")?;

        let svg = String::from_utf8(svg)?;

        Ok(svg)
    }
}

impl WriteSvg for Stroke {
    fn write_svg<W: std::io::Write>(&self, mut svg: W, scale_factor: f64) -> std::io::Result<()> {
        self.get_brush().write_svg_with_name(svg.by_ref(), "stroke")?;
        write!(svg, " stroke-width=\"{}\"", self.get_width() * scale_factor)
    }
}

impl WriteSvg for Fill {
    fn write_svg<W: std::io::Write>(&self, mut svg: W, _scale_factor: f64) -> std::io::Result<()> {
        match self {
            Fill::Color(ref color) => color.write_svg_with_name(svg, "fill"),
            Fill::Linear(ref name) => write!(svg, "fill=\"url(#{})\"", name),
            Fill::Radial(ref name) => write!(svg, "fill=\"url(#{})\"", name),
        }
    }
}

impl WriteSvg for MarkerSuit {
    fn write_svg<W: std::io::Write>(&self, mut svg: W, _scale_factor: f64) -> std::io::Result<()> {
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
    fn write_svg<W: std::io::Write>(&self, mut svg: W, scale_factor: f64) -> std::io::Result<()> {
        if let Some(stroke) = self.get_stroke() {
            stroke.write_svg(svg.by_ref(), scale_factor)?;
            write!(svg, " ")?;
        }

        if let Some(fill) = self.get_fill() {
            fill.write_svg(svg.by_ref(), scale_factor)?;
        } else {
            write!(svg, "fill=\"none\"")?;
        }

        self.get_markers().write_svg(svg, scale_factor)
    }
}

impl WriteSvgWithStyle for Crumb {
    #[inline]
    fn write_svg_with_style<W: std::io::Write>(
        &self,
        svg: W,
        ts: TranslateScale,
        style: Option<&Style>,
        theme: &Theme,
    ) -> std::io::Result<()> {
        match self {
            Crumb::Line(line) => line.write_svg_with_style(svg, ts, style, theme),
            Crumb::Rect(rect) => rect.write_svg_with_style(svg, ts, style, theme),
            Crumb::RoundedRect(rr) => rr.write_svg_with_style(svg, ts, style, theme),
            Crumb::Circle(circ) => circ.write_svg_with_style(svg, ts, style, theme),
            Crumb::Arc(arc) => arc.write_svg_with_style(svg, ts, style, theme),
            Crumb::Path(path) => path.write_svg_with_style(svg, ts, style, theme),
            Crumb::Pin(_) => Ok(()),
            Crumb::Label(label) => label.write_svg_with_style(svg, ts, style, theme),
        }
    }
}

impl WriteSvgWithStyle for Line {
    fn write_svg_with_style<W: std::io::Write>(
        &self,
        mut svg: W,
        ts: TranslateScale,
        style: Option<&Style>,
        theme: &Theme,
    ) -> std::io::Result<()> {
        let p0 = ts * self.p0;
        let p1 = ts * self.p1;
        let style = style.unwrap_or_else(|| theme.get_default_style());
        let (_, scale_factor) = ts.as_tuple();

        write!(svg, "  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" ", p0.x, p0.y, p1.x, p1.y)?;

        if let Some(stroke) = style.get_stroke() {
            stroke.write_svg(svg.by_ref(), scale_factor)?;
        }

        style.get_markers().write_svg(svg.by_ref(), scale_factor)?;

        writeln!(svg, " />")
    }
}

impl WriteSvgWithStyle for Rect {
    fn write_svg_with_style<W: std::io::Write>(
        &self,
        mut svg: W,
        ts: TranslateScale,
        style: Option<&Style>,
        _theme: &Theme,
    ) -> std::io::Result<()> {
        let rect = ts * *self;
        let (_, scale_factor) = ts.as_tuple();

        write!(
            svg,
            "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" ",
            rect.x0,
            rect.y0,
            rect.width(),
            rect.height()
        )?;

        if let Some(style) = style {
            style.write_svg(svg.by_ref(), scale_factor)?;
        }

        writeln!(svg, "/>")
    }
}

impl WriteSvgWithStyle for RoundedRect {
    fn write_svg_with_style<W: std::io::Write>(
        &self,
        mut svg: W,
        ts: TranslateScale,
        style: Option<&Style>,
        _theme: &Theme,
    ) -> std::io::Result<()> {
        let rr = ts * *self;
        let rect = &rr.rect();
        let (_, scale_factor) = ts.as_tuple();

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

        if let Some(style) = style {
            style.write_svg(svg.by_ref(), scale_factor)?;
        }

        writeln!(svg, "/>")
    }
}

impl WriteSvgWithStyle for Circle {
    fn write_svg_with_style<W: std::io::Write>(
        &self,
        mut svg: W,
        ts: TranslateScale,
        style: Option<&Style>,
        _theme: &Theme,
    ) -> std::io::Result<()> {
        let center = ts * self.center;
        let (_, scale_factor) = ts.as_tuple();
        let radius = self.radius * scale_factor;

        write!(svg, "  <circle cx=\"{}\" cy=\"{}\" r=\"{}\" ", center.x, center.y, radius)?;

        if let Some(style) = style {
            style.write_svg(svg.by_ref(), scale_factor)?;
        }

        writeln!(svg, "/>")
    }
}

impl WriteSvgWithStyle for Arc {
    fn write_svg_with_style<W: std::io::Write>(
        &self,
        svg: W,
        ts: TranslateScale,
        style: Option<&Style>,
        theme: &Theme,
    ) -> std::io::Result<()> {
        // FIXME use `A` path element
        let path = BezPath::from_vec(self.path_elements(0.1).collect());

        path.write_svg_with_style(svg, ts, style, theme)
    }
}

impl WriteSvgWithStyle for BezPath {
    fn write_svg_with_style<W: std::io::Write>(
        &self,
        mut svg: W,
        ts: TranslateScale,
        style: Option<&Style>,
        _theme: &Theme,
    ) -> std::io::Result<()> {
        let (_, scale_factor) = ts.as_tuple();

        write!(svg, "  <path d=\"")?;
        (ts * self.clone()).write_to(svg.by_ref())?;
        write!(svg, "\" ")?;

        if let Some(style) = style {
            style.write_svg(svg.by_ref(), scale_factor)?;
        }

        writeln!(svg, "/>")
    }
}

impl TextLabel {
    fn write_svg_with_scale<W: std::io::Write>(
        &self,
        mut svg: W,
        scale_factor: f64,
    ) -> std::io::Result<()> {
        if self.is_root() {
            let origin = self.get_origin().unwrap_or_default();
            write!(
                svg,
                "  <text x=\"{}\" y=\"{}\"",
                origin.x * scale_factor,
                origin.y * scale_factor
            )?;
        } else if let Some(origin) = self.get_origin() {
            write!(
                svg,
                "<tspan x=\"{}\" y=\"{}\"",
                origin.x * scale_factor,
                origin.y * scale_factor
            )?;
        } else {
            write!(svg, "<tspan")?;
        }
        match self.get_anchor() {
            crate::text::Anchor::Start => {
                if !self.is_root() {
                    write!(svg, " text-anchor=\"start\"")?;
                }
            }
            crate::text::Anchor::Middle => write!(svg, " text-anchor=\"middle\"")?,
            crate::text::Anchor::End => write!(svg, " text-anchor=\"end\"")?,
        }
        if let Some((head, tail)) = self.get_dx().split_first() {
            if tail.is_empty() {
                write!(svg, " dx=\"{}\"", head * scale_factor)?;
            } else {
                write!(svg, " dx=\"{}", head * scale_factor)?;
                for dx in tail {
                    write!(svg, " {}", dx)?;
                }
                write!(svg, "\"")?;
            }
        }
        if let Some((head, tail)) = self.get_dy().split_first() {
            if tail.is_empty() {
                write!(svg, " dy=\"{}\"", head * scale_factor)?;
            } else {
                write!(svg, " dy=\"{}", head * scale_factor)?;
                for dy in tail {
                    write!(svg, " {}", dy)?;
                }
                write!(svg, "\"")?;
            }
        }
        write!(
            svg,
            " font-family=\"{}\" font-size=\"{}\">",
            Self::DEFAULT_FONT.get_family_name(),
            Self::DEFAULT_FONT.get_size(),
        )?;

        let mut buffer = Vec::new();
        for item in self.get_body() {
            match item {
                crate::text::Item::Text(text) => {
                    svg.write(text.as_bytes())?;
                }
                crate::text::Item::Span(span) => {
                    buffer.clear();
                    span.write_svg_with_scale(&mut buffer, scale_factor)?;
                    svg.write(buffer.as_slice())?;
                }
            }
        }

        if self.is_root() {
            writeln!(svg, "</text>")
        } else {
            write!(svg, "</tspan>")
        }
    }
}

impl WriteSvgWithStyle for TextLabel {
    fn write_svg_with_style<W: std::io::Write>(
        &self,
        mut svg: W,
        ts: TranslateScale,
        style: Option<&Style>,
        theme: &Theme,
    ) -> std::io::Result<()> {
        let (_, scale_factor) = ts.as_tuple();

        if let Some(font) = self.get_font() {
            if self.is_root() {
                let origin = ts * self.get_origin().unwrap_or_default();
                write!(svg, "  <text x=\"{}\" y=\"{}\"", origin.x, origin.y,)?;
            } else if let Some(origin) = self.get_origin() {
                let origin = ts * origin;
                write!(svg, "<tspan x=\"{}\" y=\"{}\"", origin.x, origin.y,)?;
            } else {
                write!(svg, "<tspan")?;
            }
            match self.get_anchor() {
                crate::text::Anchor::Start => {
                    if !self.is_root() {
                        write!(svg, " text-anchor=\"start\"")?;
                    }
                }
                crate::text::Anchor::Middle => write!(svg, " text-anchor=\"middle\"")?,
                crate::text::Anchor::End => write!(svg, " text-anchor=\"end\"")?,
            }
            if let Some((head, tail)) = self.get_dx().split_first() {
                if tail.is_empty() {
                    write!(svg, " dx=\"{}\"", head * scale_factor)?;
                } else {
                    write!(svg, " dx=\"{}", head * scale_factor)?;
                    for dx in tail {
                        write!(svg, " {}", dx)?;
                    }
                    write!(svg, "\"")?;
                }
            }
            if let Some((head, tail)) = self.get_dy().split_first() {
                if tail.is_empty() {
                    write!(svg, " dy=\"{}\"", head * scale_factor)?;
                } else {
                    write!(svg, " dy=\"{}", head * scale_factor)?;
                    for dy in tail {
                        write!(svg, " {}", dy)?;
                    }
                    write!(svg, "\"")?;
                }
            }
            write!(
                svg,
                " font-family=\"{}\" font-size=\"{}\">",
                font.get_family_name(),
                font.get_size() * scale_factor,
            )?;

            let mut buffer = Vec::new();
            for item in self.get_body() {
                match item {
                    crate::text::Item::Text(text) => {
                        svg.write(text.as_bytes())?;
                    }
                    crate::text::Item::Span(span) => {
                        buffer.clear();
                        span.write_svg_with_style(&mut buffer, ts, style, theme)?;
                        svg.write(buffer.as_slice())?;
                    }
                }
            }

            if self.is_root() {
                writeln!(svg, "</text>")
            } else {
                writeln!(svg, "</tspan>")
            }
        } else {
            // FIXME translation?
            self.write_svg_with_scale(svg, scale_factor)
        }
    }
}

impl WriteSvgWithName for Color {
    fn write_svg_with_name<W: std::io::Write, S: AsRef<str>>(
        &self,
        mut svg: W,
        name: S,
    ) -> std::io::Result<()> {
        let rgba = self.as_u32();
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

impl WriteSvgWithName for Gradient {
    fn write_svg_with_name<W: std::io::Write, S: AsRef<str>>(
        &self,
        mut svg: W,
        name: S,
    ) -> std::io::Result<()> {
        match self {
            Gradient::Linear(start, end, stops) => {
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
                    stop.color.write_svg_with_name(svg.by_ref(), "stop-color")?;
                    writeln!(svg, "/>")?;
                }

                writeln!(svg, "    </linearGradient>")
            }
            Gradient::Radial(radius, stops) => {
                writeln!(
                    svg,
                    "    <radialGradient id=\"{}\" r=\"{}%\">",
                    name.as_ref(),
                    *radius * 100.
                )?;

                for stop in stops.iter() {
                    write!(svg, "      <stop offset=\"{}\" ", stop.pos)?;
                    stop.color.write_svg_with_name(svg.by_ref(), "stop-color")?;
                    writeln!(svg, "/>")?;
                }

                writeln!(svg, "    </radialGradient>")
            }
        }
    }
}

impl WriteSvgWithTheme for Marker {
    fn write_svg_with_theme<W: std::io::Write, S: AsRef<str>>(
        &self,
        mut svg: W,
        scale_factor: f64,
        name: S,
        theme: &Theme,
    ) -> std::io::Result<()> {
        write!(svg, "    <marker id=\"{}\" ", name.as_ref())?;
        if let Some(orient) = self.get_orient() {
            writeln!(svg, "orient=\"{}\"", orient)?;
        } else {
            writeln!(svg, "orient=\"auto\"")?;
        }
        writeln!(
            svg,
            "            markerUnits=\"userSpaceOnUse\" markerWidth=\"{}\" markerHeight=\"{}\"",
            self.get_width() * scale_factor,
            self.get_height() * scale_factor,
        )?;
        writeln!(
            svg,
            "            refX=\"{}\" refY=\"{}\">",
            self.get_refx() * scale_factor,
            self.get_refy() * scale_factor
        )?;

        self.get_crumb().write_svg_with_style(
            svg.by_ref(),
            TranslateScale::scale(scale_factor),
            self.get_style_name().and_then(|name| theme.get_style_by_name(name)),
            theme,
        )?;

        writeln!(svg, "    </marker>")
    }
}
