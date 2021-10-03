use kurbo::{Line, Rect, RoundedRect, Circle, Arc, BezPath, TranslateScale};
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
