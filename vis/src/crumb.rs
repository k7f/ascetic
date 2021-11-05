use std::f64::consts::PI;
use kurbo::{Point, Line, Rect, RoundedRect, Circle, Arc, BezPath, TranslateScale};
use crate::{Scene, StyleId, TextLabel, VisError};

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

macro_rules! impl_into_crumb {
    ($shape:ty, $variant:ident) => {
        impl From<$shape> for Crumb {
            #[inline]
            fn from(shape: $shape) -> Self {
                Crumb::$variant(shape)
            }
        }
    };
}

impl_into_crumb!(Line, Line);
impl_into_crumb!(Rect, Rect);
impl_into_crumb!(RoundedRect, RoundedRect);
impl_into_crumb!(Circle, Circle);
impl_into_crumb!(Arc, Arc);
impl_into_crumb!(BezPath, Path);
impl_into_crumb!(TextLabel, Label);

pub trait Crumbling: Into<Crumb> {
    fn end_angle(&self, points: &[Point]) -> f64 {
        if let Some((p1, head)) = points.split_last() {
            if let Some(p0) = head.last() {
                let angle = (p1.y - p0.y).atan2(p1.x - p0.x);

                if !angle.is_nan() {
                    if angle < 0.0 {
                        return angle % (PI * 2.0) + PI * 2.0
                    } else {
                        return angle % (PI * 2.0)
                    }
                }
            }
        }
        0.0
    }
}

impl Crumbling for Crumb {
    #[inline]
    fn end_angle(&self, points: &[Point]) -> f64 {
        match self {
            Crumb::Line(line) => line.end_angle(points),
            Crumb::Rect(rect) => rect.end_angle(points),
            Crumb::RoundedRect(rr) => rr.end_angle(points),
            Crumb::Circle(circ) => circ.end_angle(points),
            Crumb::Arc(arc) => arc.end_angle(points),
            Crumb::Path(path) => path.end_angle(points),
            Crumb::Pin(pin) => pin.end_angle(points),
            Crumb::Label(label) => label.end_angle(points),
        }
    }
}

impl Crumbling for Line {
    fn end_angle(&self, _points: &[Point]) -> f64 {
        let angle = (self.p1.y - self.p0.y).atan2(self.p1.x - self.p0.x);

        if angle.is_nan() {
            0.0
        } else if angle < 0.0 {
            angle % (PI * 2.0) + PI * 2.0
        } else {
            angle % (PI * 2.0)
        }
    }
}

impl Crumbling for Rect {}
impl Crumbling for RoundedRect {}
impl Crumbling for Circle {}
impl Crumbling for Arc {}
impl Crumbling for BezPath {}
impl Crumbling for TextLabel {}

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
        F: FnMut(&mut TextLabel, TranslateScale, Option<StyleId>) -> Result<(), VisError>,
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
