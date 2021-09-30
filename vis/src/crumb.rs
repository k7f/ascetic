use piet::RenderContext;
use kurbo::{Shape, Line, Rect, RoundedRect, Circle, Arc, BezPath, TranslateScale};
use crate::{Vis, Scene, Theme, Style, StyleId, TextLabel, VisError};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CrumbId(pub usize);

#[derive(Clone, Copy, Debug)]
pub struct CrumbItem(pub CrumbId, pub TranslateScale, pub Option<StyleId>);

#[derive(Clone, Debug)]
pub enum Crumb {
    Line(Line),
    Rect(Rect),
    RoundedRect(RoundedRect),
    Circle(Circle),
    Arc(Arc),
    Path(BezPath),
    Pin(Circle),
    Label(TextLabel),
}

#[derive(Clone, Default, Debug)]
pub struct CrumbSet {
    crumbs: Vec<CrumbItem>,
    levels: Vec<usize>,
}

impl CrumbSet {
    pub fn get_crumbs<'a>(&'a self, scene: &'a Scene) -> CrumbSetIter<'a> {
        CrumbSetIter { scene, items: self.crumbs.iter() }
    }

    pub fn try_for_each_label<F>(&self, scene: &mut Scene, mut f: F) -> Result<(), VisError>
    where
        F: FnMut(&mut TextLabel, TranslateScale, Option<StyleId>) -> Result<(), VisError>
    {
        for CrumbItem(crumb_id, ts, style_id) in &self.crumbs {
            if let Some(crumb) = scene.get_crumb_mut(*crumb_id) {
                if let Crumb::Label(label) = crumb {
                    f(label, *ts, *style_id)?;
                }
            } else {
                return Err(VisError::crumb_missing_for_id(*crumb_id))
            }
        }
        Ok(())
    }
}

impl std::iter::FromIterator<(usize, CrumbItem)> for CrumbSet {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (usize, CrumbItem)>,
    {
        let mut result = CrumbSet::default();

        for (level, item) in iter {
            result.crumbs.push(item);
            result.levels.push(level);
        }

        result
    }
}

pub struct CrumbSetIter<'a> {
    scene: &'a Scene,
    items: std::slice::Iter<'a, CrumbItem>,
}

impl<'a> Iterator for CrumbSetIter<'a> {
    type Item = Result<(&'a Crumb, TranslateScale, Option<StyleId>), VisError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        #[allow(clippy::collapsible_match)]
        if let Some(CrumbItem(crumb_id, ts, style_id)) = self.items.next() {
            if let Some(crumb) = self.scene.get_crumb(*crumb_id) {
                Some(Ok((crumb, *ts, *style_id)))
            } else {
                Some(Err(VisError::crumb_missing_for_id(*crumb_id)))
            }
        } else {
            None
        }
    }
}

impl<R: RenderContext> Vis<R> for Line {
    #[inline]
    fn bbox(&self, _rc: &mut R, ts: TranslateScale) -> Rect {
        (ts * *self).bounding_box()
    }

    fn vis(&self, rc: &mut R, ts: TranslateScale, style: Option<&Style>, theme: &Theme) {
        if let Some(stroke) =
            style.and_then(|s| s.get_stroke()).or_else(|| theme.get_default_style().get_stroke())
        {
            rc.stroke(ts * *self, stroke.get_brush(), stroke.get_width());
        }
    }
}

impl<R: RenderContext> Vis<R> for Rect {
    #[inline]
    fn bbox(&self, _rc: &mut R, ts: TranslateScale) -> Rect {
        (ts * *self).bounding_box()
    }

    fn vis(&self, rc: &mut R, ts: TranslateScale, style: Option<&Style>, theme: &Theme) {
        let style = style.unwrap_or_else(|| theme.get_default_style());
        let rect = ts * *self;

        if let Some(brush) = style.get_fill_color() {
            rc.fill(rect, brush);
        } else if let Some(name) = style.get_fill_gradient_name() {
            if let Some(gradient) = theme.get_linear_gradient(name) {
                rc.fill(rect, gradient);
            } else if let Some(gradient) = theme.get_radial_gradient(name) {
                rc.fill(rect, gradient);
            }
        }

        if let Some(border) = style.get_stroke() {
            rc.stroke(rect, border.get_brush(), border.get_width());
        }
    }
}

impl<R: RenderContext> Vis<R> for RoundedRect {
    #[inline]
    fn bbox(&self, _rc: &mut R, ts: TranslateScale) -> Rect {
        (ts * *self).bounding_box()
    }

    fn vis(&self, rc: &mut R, ts: TranslateScale, style: Option<&Style>, theme: &Theme) {
        let style = style.unwrap_or_else(|| theme.get_default_style());
        let rr = ts * *self;

        if let Some(brush) = style.get_fill_color() {
            rc.fill(rr, brush);
        } else if let Some(name) = style.get_fill_gradient_name() {
            if let Some(gradient) = theme.get_linear_gradient(name) {
                rc.fill(rr, gradient);
            } else if let Some(gradient) = theme.get_radial_gradient(name) {
                rc.fill(rr, gradient);
            }
        }

        if let Some(border) = style.get_stroke() {
            rc.stroke(rr, border.get_brush(), border.get_width());
        }
    }
}

impl<R: RenderContext> Vis<R> for Circle {
    #[inline]
    fn bbox(&self, _rc: &mut R, ts: TranslateScale) -> Rect {
        (ts * *self).bounding_box()
    }

    fn vis(&self, rc: &mut R, ts: TranslateScale, style: Option<&Style>, theme: &Theme) {
        let style = style.unwrap_or_else(|| theme.get_default_style());
        let circ = ts * *self;

        if let Some(brush) = style.get_fill_color() {
            rc.fill(circ, brush);
        } else if let Some(name) = style.get_fill_gradient_name() {
            if let Some(gradient) = theme.get_linear_gradient(name) {
                rc.fill(circ, gradient);
            } else if let Some(gradient) = theme.get_radial_gradient(name) {
                rc.fill(circ, gradient);
            }
        }

        if let Some(border) = style.get_stroke() {
            rc.stroke(circ, border.get_brush(), border.get_width());
        }
    }
}

impl<R: RenderContext> Vis<R> for Arc {
    #[inline]
    fn bbox(&self, _rc: &mut R, ts: TranslateScale) -> Rect {
        ts * self.bounding_box()
    }

    fn vis(&self, rc: &mut R, ts: TranslateScale, style: Option<&Style>, theme: &Theme) {
        let style = style.unwrap_or_else(|| theme.get_default_style());
        let path = ts * BezPath::from_vec(self.path_elements(0.1).collect());

        if let Some(brush) = style.get_fill_color() {
            rc.fill(&path, brush);
        } else if let Some(name) = style.get_fill_gradient_name() {
            if let Some(gradient) = theme.get_linear_gradient(name) {
                rc.fill(&path, gradient);
            } else if let Some(gradient) = theme.get_radial_gradient(name) {
                rc.fill(&path, gradient);
            }
        }

        if let Some(border) = style.get_stroke() {
            rc.stroke(&path, border.get_brush(), border.get_width());
        }
    }
}

impl<R: RenderContext> Vis<R> for BezPath {
    #[inline]
    fn bbox(&self, _rc: &mut R, ts: TranslateScale) -> Rect {
        ts * self.bounding_box()
    }

    fn vis(&self, rc: &mut R, ts: TranslateScale, style: Option<&Style>, theme: &Theme) {
        let style = style.unwrap_or_else(|| theme.get_default_style());
        let path = ts * self.clone();

        if let Some(brush) = style.get_fill_color() {
            rc.fill(&path, brush);
        } else if let Some(name) = style.get_fill_gradient_name() {
            if let Some(gradient) = theme.get_linear_gradient(name) {
                rc.fill(&path, gradient);
            } else if let Some(gradient) = theme.get_radial_gradient(name) {
                rc.fill(&path, gradient);
            }
        }

        if let Some(border) = style.get_stroke() {
            rc.stroke(&path, border.get_brush(), border.get_width());
        }
    }
}
