use rgb::ComponentBytes;
use kurbo::{Point, Line, Rect, RoundedRect, Circle, Arc, BezPath, Shape, TranslateScale, Size};
use femtovg as fvg;
use crate::{
    Scene, Theme, Style, StyleId, Stroke, Fill, Color, GradientStop, Gradient, Crumb, CrumbItem,
    Crumbling, TextLabel, VisError,
};

pub trait Renderable<T: fvg::Renderer> {
    fn render_as_fvg<S, M>(
        &self,
        canvas: &mut fvg::Canvas<T>,
        theme: &Theme,
        out_size: S,
        out_margin: M,
    ) -> Result<(), VisError>
    where
        S: Into<Size>,
        M: Into<Size>;
}

impl<T: fvg::Renderer> Renderable<T> for Scene {
    fn render_as_fvg<S, M>(
        &self,
        canvas: &mut fvg::Canvas<T>,
        theme: &Theme,
        out_size: S,
        out_margin: M,
    ) -> Result<(), VisError>
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

        canvas.set_size(out_size.width as u32, out_size.height as u32, 1.0);

        theme.render_background_as_fvg(canvas);

        for (_level, CrumbItem(crumb_id, ts, style_id)) in self.all_visible_crumbs(root_ts)? {
            if let Some(crumb) = self.get_crumb(crumb_id) {
                crumb.render_as_fvg_with_style(canvas, ts, style_id, theme)?;
            } else {
                return Err(VisError::crumb_missing_for_id(crumb_id))
            }
        }

        // let mut path = fvg::Path::new();
        // path.rect(0.0, 0.0, canvas.width(), canvas.height());
        // canvas.fill_path(&mut path, ...as_fvg_paint());

        Ok(())
    }
}

pub trait AsPath {
    fn as_fvg_path(&self, ts: TranslateScale) -> fvg::Path;

    fn as_fvg_path_and_points(&self, ts: TranslateScale) -> (fvg::Path, Vec<Point>) {
        (self.as_fvg_path(ts), Vec::new())
    }
}

impl AsPath for Crumb {
    #[inline]
    fn as_fvg_path(&self, ts: TranslateScale) -> fvg::Path {
        match self {
            Crumb::Line(line) => line.as_fvg_path(ts),
            Crumb::Rect(rect) => rect.as_fvg_path(ts),
            Crumb::RoundedRect(rr) => rr.as_fvg_path(ts),
            Crumb::Circle(circ) => circ.as_fvg_path(ts),
            Crumb::Arc(arc) => arc.as_fvg_path(ts),
            Crumb::Path(path) => path.as_fvg_path(ts),
            Crumb::Pin(_) => fvg::Path::new(),
            Crumb::Label(label) => label.as_fvg_path(ts),
        }
    }

    #[inline]
    fn as_fvg_path_and_points(&self, ts: TranslateScale) -> (fvg::Path, Vec<Point>) {
        match self {
            Crumb::Line(line) => line.as_fvg_path_and_points(ts),
            Crumb::Rect(rect) => rect.as_fvg_path_and_points(ts),
            Crumb::RoundedRect(rr) => rr.as_fvg_path_and_points(ts),
            Crumb::Circle(circ) => circ.as_fvg_path_and_points(ts),
            Crumb::Arc(arc) => arc.as_fvg_path_and_points(ts),
            Crumb::Path(path) => path.as_fvg_path_and_points(ts),
            Crumb::Pin(_) => (fvg::Path::new(), Vec::new()),
            Crumb::Label(label) => label.as_fvg_path_and_points(ts),
        }
    }
}

impl AsPath for Line {
    fn as_fvg_path(&self, ts: TranslateScale) -> fvg::Path {
        self.as_fvg_path_and_points(ts).0
    }

    fn as_fvg_path_and_points(&self, ts: TranslateScale) -> (fvg::Path, Vec<Point>) {
        let p0 = ts * self.p0;
        let p1 = ts * self.p1;
        let mut path = fvg::Path::new();

        path.move_to(p0.x as f32, p0.y as f32);
        path.line_to(p1.x as f32, p1.y as f32);

        (path, vec![p0, p1])
    }
}

impl AsPath for Rect {
    fn as_fvg_path(&self, ts: TranslateScale) -> fvg::Path {
        let rect = ts * *self;
        let mut path = fvg::Path::new();

        path.rect(rect.x0 as f32, rect.y0 as f32, rect.width() as f32, rect.height() as f32);

        path
    }
}

impl AsPath for RoundedRect {
    fn as_fvg_path(&self, ts: TranslateScale) -> fvg::Path {
        let rr = ts * *self;

        if let Some(radius) = rr.radii().as_single_radius() {
            let rect = rr.rect();
            let mut path = fvg::Path::new();

            path.rounded_rect(
                rect.x0 as f32,
                rect.y0 as f32,
                rect.width() as f32,
                rect.height() as f32,
                radius as f32,
            );

            path
        } else {
            self.rect().as_fvg_path(ts)
        }
    }
}

impl AsPath for Circle {
    fn as_fvg_path(&self, ts: TranslateScale) -> fvg::Path {
        let center = ts * self.center;
        let (cx, cy) = (center.x as f32, center.y as f32);
        let radius = (ts.as_tuple().1 * self.radius) as f32;
        let mut path = fvg::Path::new();

        path.circle(cx, cy, radius);

        path
    }
}

impl AsPath for Arc {
    #[inline]
    fn as_fvg_path(&self, ts: TranslateScale) -> fvg::Path {
        let bez_path = BezPath::from_vec(self.path_elements(0.1).collect());

        bez_path.as_fvg_path(ts)
    }

    #[inline]
    fn as_fvg_path_and_points(&self, ts: TranslateScale) -> (fvg::Path, Vec<Point>) {
        let bez_path = BezPath::from_vec(self.path_elements(0.1).collect());

        bez_path.as_fvg_path_and_points(ts)
    }
}

impl AsPath for BezPath {
    #[inline]
    fn as_fvg_path(&self, ts: TranslateScale) -> fvg::Path {
        self.as_fvg_path_and_points(ts).0
    }

    fn as_fvg_path_and_points(&self, ts: TranslateScale) -> (fvg::Path, Vec<Point>) {
        let mut out_path = fvg::Path::new();
        let mut out_points = Vec::new();

        for path_el in self.iter() {
            use kurbo::PathEl::*;
            match path_el {
                MoveTo(point) => {
                    let point = ts * point;
                    out_path.move_to(point.x as f32, point.y as f32);
                    out_points.push(point);
                }
                LineTo(point) => {
                    let point = ts * point;
                    out_path.line_to(point.x as f32, point.y as f32);
                    out_points.push(point);
                }
                QuadTo(point1, point2) => {
                    let point1 = ts * point1;
                    let point2 = ts * point2;
                    out_path.quad_to(
                        point1.x as f32,
                        point1.y as f32,
                        point2.x as f32,
                        point2.y as f32,
                    );
                    out_points.push(point1);
                    out_points.push(point2);
                }
                CurveTo(point1, point2, point3) => {
                    let point1 = ts * point1;
                    let point2 = ts * point2;
                    let point3 = ts * point3;
                    out_path.bezier_to(
                        point1.x as f32,
                        point1.y as f32,
                        point2.x as f32,
                        point2.y as f32,
                        point3.x as f32,
                        point3.y as f32,
                    );
                    out_points.push(point1);
                    out_points.push(point2);
                    out_points.push(point3);
                }
                ClosePath => out_path.close(),
            }
        }

        (out_path, out_points)
    }
}

impl AsPath for TextLabel {
    fn as_fvg_path(&self, ts: TranslateScale) -> fvg::Path {
        fvg::Path::new()
    }
}

pub trait RenderableWithStyle<T: fvg::Renderer>: Crumbling + AsPath {
    fn render_as_fvg_with_style(
        &self,
        canvas: &mut fvg::Canvas<T>,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> Result<(), VisError> {
        let mut path = self.as_fvg_path(ts);

        if !path.is_empty() {
            let style = theme.get_style(style_id).unwrap_or_else(|| theme.get_default_style());
            let fill_paint = style
                .get_fill()
                .or_else(|| theme.get_default_style().get_fill())
                .map(|f| f.as_fvg_paint_with_theme(theme));
            let stroke_paint = style
                .get_stroke()
                .or_else(|| theme.get_default_style().get_stroke())
                .map(|s| s.as_fvg_paint());

            if let Some(paint) = fill_paint {
                canvas.fill_path(&mut path, paint?);
            }

            if let Some(paint) = stroke_paint {
                canvas.stroke_path(&mut path, paint);
            }
        }

        Ok(())
    }
}

impl<T: fvg::Renderer> RenderableWithStyle<T> for Crumb {
    #[inline]
    fn render_as_fvg_with_style(
        &self,
        canvas: &mut fvg::Canvas<T>,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> Result<(), VisError> {
        match self {
            Crumb::Line(line) => line.render_as_fvg_with_style(canvas, ts, style_id, theme),
            Crumb::Rect(rect) => rect.render_as_fvg_with_style(canvas, ts, style_id, theme),
            Crumb::RoundedRect(rr) => rr.render_as_fvg_with_style(canvas, ts, style_id, theme),
            Crumb::Circle(circ) => circ.render_as_fvg_with_style(canvas, ts, style_id, theme),
            Crumb::Arc(arc) => arc.render_as_fvg_with_style(canvas, ts, style_id, theme),
            Crumb::Path(path) => path.render_as_fvg_with_style(canvas, ts, style_id, theme),
            Crumb::Pin(_) => Ok(()),
            Crumb::Label(label) => label.render_as_fvg_with_style(canvas, ts, style_id, theme),
        }
    }
}

impl<T: fvg::Renderer> RenderableWithStyle<T> for Line {
    fn render_as_fvg_with_style(
        &self,
        canvas: &mut fvg::Canvas<T>,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> Result<(), VisError> {
        let style = theme.get_style(style_id).unwrap_or_else(|| theme.get_default_style());
        let stroke = style.get_stroke().or_else(|| theme.get_default_style().get_stroke());
        let paint = if let Some(stroke) = stroke {
            stroke.as_fvg_paint()
        } else {
            let stroke = Stroke::default();
            stroke.as_fvg_paint()
        };
        let mut path = {
            let markers = style.get_markers();

            if let Some(marker) =
                markers.get_end_name().and_then(|name| theme.get_marker_by_name(name))
            {
                let (path, points) = self.as_fvg_path_and_points(ts);

                // FIXME precompute marker's path, clone it here.
                let mut marker_path = marker.get_crumb().as_fvg_path(TranslateScale::default());

                // FIXME precompute unit width.
                let unit_width = stroke.as_ref().map(|s| s.get_width()).unwrap_or(1.0);
                let angle =
                    marker.get_orient().unwrap_or_else(|| self.end_angle(points.as_slice()));
                let a = angle.cos() * unit_width;
                let b = angle.sin() * unit_width;
                let refx = marker.get_refx();
                let refy = marker.get_refy();
                let p1 = points.last().unwrap();

                path
            } else {
                self.as_fvg_path(ts)
            }
        };

        canvas.stroke_path(&mut path, paint);

        Ok(())
    }
}

impl<T: fvg::Renderer> RenderableWithStyle<T> for Rect {}
impl<T: fvg::Renderer> RenderableWithStyle<T> for RoundedRect {}
impl<T: fvg::Renderer> RenderableWithStyle<T> for Circle {}

impl<T: fvg::Renderer> RenderableWithStyle<T> for Arc {
    fn render_as_fvg_with_style(
        &self,
        canvas: &mut fvg::Canvas<T>,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> Result<(), VisError> {
        Ok(())
    }
}

impl<T: fvg::Renderer> RenderableWithStyle<T> for BezPath {
    fn render_as_fvg_with_style(
        &self,
        canvas: &mut fvg::Canvas<T>,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> Result<(), VisError> {
        Ok(())
    }
}

impl<T: fvg::Renderer> RenderableWithStyle<T> for TextLabel {
    fn render_as_fvg_with_style(
        &self,
        canvas: &mut fvg::Canvas<T>,
        ts: TranslateScale,
        style_id: Option<StyleId>,
        theme: &Theme,
    ) -> Result<(), VisError> {
        Ok(())
    }
}

pub trait AsPaint {
    fn as_fvg_paint(&self) -> fvg::Paint;
}

impl AsPaint for Stroke {
    fn as_fvg_paint(&self) -> fvg::Paint {
        let mut paint = fvg::Paint::color(self.get_brush().as_fvg_color());

        paint.set_line_width(self.get_width() as f32);

        paint
    }
}

pub trait AsPaintWithTheme {
    fn as_fvg_paint_with_theme(&self, theme: &Theme) -> Result<fvg::Paint, VisError>;
}

impl AsPaintWithTheme for Fill {
    fn as_fvg_paint_with_theme(&self, theme: &Theme) -> Result<fvg::Paint, VisError> {
        match self {
            Fill::Color(color) => Ok(fvg::Paint::color(color.as_fvg_color())),
            Fill::Linear(name) => match theme.get_gradspec(name) {
                Some(Gradient::Linear(start, end, stops)) => {
                    let start = start.resolve(Rect::new(0., 0., 1., 1.));
                    let end = end.resolve(Rect::new(0., 0., 1., 1.));
                    let stops: Vec<_> =
                        stops.iter().map(|stop| (stop.pos, stop.color.as_fvg_color())).collect();

                    Ok(fvg::Paint::linear_gradient_stops(
                        start.x as f32,
                        start.y as f32,
                        end.x as f32,
                        end.y as f32,
                        stops.as_slice(),
                    ))
                }
                Some(_) => Err(VisError::gradient_mismatch_for_name(name)),
                None => Err(VisError::gradient_missing_for_name(name)),
            },
            Fill::Radial(name) => match theme.get_gradspec(name) {
                Some(Gradient::Radial(radius, stops)) => {
                    let radius = *radius as f32;
                    let stops: Vec<_> =
                        stops.iter().map(|stop| (stop.pos, stop.color.as_fvg_color())).collect();

                    Ok(fvg::Paint::radial_gradient_stops(
                        0.0,
                        0.0,
                        radius,
                        radius,
                        stops.as_slice(),
                    ))
                }
                Some(_) => Err(VisError::gradient_mismatch_for_name(name)),
                None => Err(VisError::gradient_missing_for_name(name)),
            },
        }
    }
}

pub trait AsColor {
    fn as_fvg_color(self) -> fvg::Color;
}

impl AsColor for Color {
    #[inline]
    fn as_fvg_color(self) -> fvg::Color {
        let (r, g, b, a) = self.as_rgba8();

        if a == 0xff {
            fvg::Color::rgb(r, g, b)
        } else {
            fvg::Color::rgba(r, g, b, a)
        }
    }
}

pub trait RenderableTheme<T: fvg::Renderer> {
    fn render_background_as_fvg(&self, canvas: &mut fvg::Canvas<T>);
}

impl<T: fvg::Renderer> RenderableTheme<T> for Theme {
    fn render_background_as_fvg(&self, canvas: &mut fvg::Canvas<T>) {
        let (r, g, b, a) = self.get_bg_color().as_rgba8();

        canvas.clear_rect(
            0,
            0,
            canvas.width() as u32,
            canvas.height() as u32,
            self.get_bg_color().as_fvg_color(),
        );
    }
}
