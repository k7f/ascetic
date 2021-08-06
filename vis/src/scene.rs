use std::{slice, error::Error};
use piet::RenderContext;
use kurbo::{Point, Line, Rect, RoundedRect, Circle, Arc, BezPath, PathEl, TranslateScale, Size};
use crate::{Vis, Crumb, CrumbId, CrumbItem, Group, GroupId, GroupItem, StyleId, Theme};

#[derive(Clone, Default, Debug)]
pub struct Scene {
    size:   Size,
    crumbs: Vec<Crumb>,
    groups: Vec<Group>,
    roots:  Vec<GroupId>,
}

impl Scene {
    pub fn new<S: Into<Size>>(size: S) -> Self {
        Scene { size: size.into(), ..Default::default() }
    }

    #[inline]
    pub fn get_size(&self) -> &Size {
        &self.size
    }

    #[inline]
    pub fn get_crumb(&self, crumb_id: CrumbId) -> Option<&Crumb> {
        self.crumbs.get(crumb_id.0)
    }

    #[inline]
    pub fn get_group(&self, group_id: GroupId) -> Option<&Group> {
        self.groups.get(group_id.0)
    }

    pub fn add_line(&mut self, line: Line) -> CrumbId {
        let id = self.crumbs.len();

        self.crumbs.push(Crumb::Line(line));

        CrumbId(id)
    }

    pub fn add_rect(&mut self, rect: Rect) -> CrumbId {
        let id = self.crumbs.len();

        self.crumbs.push(Crumb::Rect(rect));

        CrumbId(id)
    }

    pub fn add_rounded_rect(&mut self, rect: RoundedRect) -> CrumbId {
        let id = self.crumbs.len();

        self.crumbs.push(Crumb::RoundedRect(rect));

        CrumbId(id)
    }

    pub fn add_circle(&mut self, circ: Circle) -> CrumbId {
        let id = self.crumbs.len();

        self.crumbs.push(Crumb::Circle(circ));

        CrumbId(id)
    }

    pub fn add_crumb(&mut self, crumb: Crumb) -> CrumbId {
        let id = self.crumbs.len();

        self.crumbs.push(crumb);

        CrumbId(id)
    }

    pub fn add_group(&mut self, group: Group) -> GroupId {
        let id = self.groups.len();

        self.groups.push(group);

        GroupId(id)
    }

    pub fn add_grouped_crumbs<I>(&mut self, crumbs: I) -> GroupId
    where
        I: IntoIterator<Item = (Crumb, Option<StyleId>)>,
    {
        let crumbs = crumbs.into_iter().map(|(p, s)| (self.add_crumb(p), s));
        let group = Group::from_crumbs(crumbs);

        self.add_group(group)
    }

    pub fn add_grouped_crumb_items<I>(&mut self, crumbs: I) -> GroupId
    where
        I: IntoIterator<Item = CrumbItem>,
    {
        let group = Group::from_crumb_items(crumbs.into_iter());

        self.add_group(group)
    }

    pub fn add_grouped_lines<I>(&mut self, lines: I) -> GroupId
    where
        I: IntoIterator<Item = (Line, Option<StyleId>)>,
    {
        let crumbs = lines.into_iter().map(|(l, s)| (self.add_line(l), s));
        let group = Group::from_crumbs(crumbs);

        self.add_group(group)
    }

    pub fn add_root(&mut self, group: Group) -> GroupId {
        let group_id = GroupId(self.groups.len());

        self.groups.push(group);
        self.roots.push(group_id);

        group_id
    }

    pub fn line_joint(
        &self,
        theme: &Theme,
        group_id: GroupId,
        tail: usize,
        head: usize,
    ) -> Option<Line> {
        if let Some(group) = self.get_group(group_id) {
            let items = group.get_crumb_items();
            if let Some(CrumbItem(tail_id, ..)) = items.get(tail) {
                if let Some(CrumbItem(head_id, ..)) = items.get(head) {
                    if let Some(tail_crumb) = self.get_crumb(*tail_id) {
                        if let Some(head_crumb) = self.get_crumb(*head_id) {
                            let (tail_p0, tail_r) = match tail_crumb {
                                Crumb::Circle(c) => (c.center, c.radius),
                                Crumb::Pin(p) => (p.center, p.radius),
                                _ => return None,
                            };
                            let (head_p0, head_r) = match head_crumb {
                                Crumb::Circle(c) => (c.center, c.radius),
                                Crumb::Pin(p) => (p.center, p.radius),
                                _ => return None,
                            };

                            let versor = (head_p0 - tail_p0) / (head_p0 - tail_p0).hypot();

                            let mut marker_len = theme
                                .get_marker_by_name("arrowhead1")
                                .map(|m| m.get_width())
                                .unwrap_or(0.0);
                            if let Some(stroke) = theme.get_stroke_by_name("line-thin") {
                                marker_len += 2.0 * stroke.get_width();
                            }

                            let border_width = theme
                                .get_stroke_by_name("node")
                                .map(|s| s.get_width())
                                .unwrap_or(0.0);

                            let tail_p1 = tail_p0 + versor * (tail_r + border_width);
                            let head_p1 = head_p0 - versor * (head_r + marker_len + border_width);

                            return Some(Line::new(tail_p1, head_p1))
                        }
                    }
                }
            }
        }
        None
    }

    // FIXME line2_joint, line3_joint

    pub fn arc_joint(
        &self,
        theme: &Theme,
        group_id: GroupId,
        tail: usize,
        head: usize,
        mut radius: f64,
    ) -> Option<Arc> {
        if let Some(group) = self.get_group(group_id) {
            let items = group.get_crumb_items();
            if let Some(CrumbItem(tail_id, ..)) = items.get(tail) {
                if let Some(CrumbItem(head_id, ..)) = items.get(head) {
                    if let Some(tail_crumb) = self.get_crumb(*tail_id) {
                        if let Some(head_crumb) = self.get_crumb(*head_id) {
                            let (tail_p0, tail_r) = match tail_crumb {
                                Crumb::Circle(c) => (c.center, c.radius),
                                Crumb::Pin(p) => (p.center, p.radius),
                                _ => return None,
                            };
                            let (head_p0, head_r) = match head_crumb {
                                Crumb::Circle(c) => (c.center, c.radius),
                                Crumb::Pin(p) => (p.center, p.radius),
                                _ => return None,
                            };

                            let mid_x = (head_p0.x + tail_p0.x) * 0.5;
                            let mid_y = (head_p0.y + tail_p0.y) * 0.5;
                            let halfdist = (head_p0 - tail_p0).hypot() * 0.5;

                            if halfdist < tail_r + head_r + 0.1 {
                                return None
                            }

                            let center: Point = if radius > halfdist {
                                let base = (radius * radius - halfdist * halfdist).sqrt()
                                    / (halfdist * 2.0);
                                (
                                    mid_x + (tail_p0.y - head_p0.y) * base,
                                    mid_y + (tail_p0.x - head_p0.x) * base,
                                )
                            } else if radius < -halfdist {
                                radius = -radius;
                                let base = (radius * radius - halfdist * halfdist).sqrt()
                                    / (halfdist * 2.0);
                                (
                                    mid_x - (tail_p0.y - head_p0.y) * base,
                                    mid_y - (tail_p0.x - head_p0.x) * base,
                                )
                            } else {
                                // FIXME line segment
                                return None
                            }
                            .into();

                            let mut marker_len = theme
                                .get_marker_by_name("arrowhead1")
                                .map(|m| m.get_width())
                                .unwrap_or(0.0);
                            if let Some(stroke) = theme.get_stroke_by_name("line-thin") {
                                marker_len += 2.0 * stroke.get_width();
                            }

                            let border_width = theme
                                .get_stroke_by_name("node")
                                .map(|s| s.get_width())
                                .unwrap_or(0.0);

                            let tail_apex_angle =
                                2.0 * ((tail_r + border_width) / (2.0 * radius)).asin();
                            let head_apex_angle = 2.0
                                * ((head_r + marker_len + border_width) / (2.0 * radius)).asin();
                            let total_apex_angle = tail_apex_angle + head_apex_angle;

                            let start_angle = (tail_p0.y - center.y).atan2(tail_p0.x - center.x);
                            let mut sweep_angle =
                                (head_p0.y - center.y).atan2(head_p0.x - center.x) - start_angle;

                            if sweep_angle >= std::f64::consts::PI {
                                sweep_angle -= 2.0 * std::f64::consts::PI;
                            } else if sweep_angle <= -std::f64::consts::PI {
                                sweep_angle += 2.0 * std::f64::consts::PI;
                            }

                            if sweep_angle > total_apex_angle {
                                return Some(Arc {
                                    center,
                                    radii: (radius, radius).into(),
                                    start_angle: start_angle + tail_apex_angle,
                                    sweep_angle: sweep_angle - total_apex_angle,
                                    x_rotation: 0.0,
                                })
                            } else if sweep_angle < -total_apex_angle {
                                return Some(Arc {
                                    center,
                                    radii: (radius, radius).into(),
                                    start_angle: start_angle - tail_apex_angle,
                                    sweep_angle: sweep_angle + total_apex_angle,
                                    x_rotation: 0.0,
                                })
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub fn quad_joint(
        &self,
        theme: &Theme,
        group_id: GroupId,
        tail: usize,
        head: usize,
        bendx: f64,
        bendy: f64,
    ) -> Option<BezPath> {
        if let Some(group) = self.get_group(group_id) {
            let items = group.get_crumb_items();
            if let Some(CrumbItem(tail_id, ..)) = items.get(tail) {
                if let Some(CrumbItem(head_id, ..)) = items.get(head) {
                    if let Some(tail_crumb) = self.get_crumb(*tail_id) {
                        if let Some(head_crumb) = self.get_crumb(*head_id) {
                            let (tail_p0, tail_r) = match tail_crumb {
                                Crumb::Circle(c) => (c.center, c.radius),
                                Crumb::Pin(p) => (p.center, p.radius),
                                _ => return None,
                            };
                            let (head_p0, head_r) = match head_crumb {
                                Crumb::Circle(c) => (c.center, c.radius),
                                Crumb::Pin(p) => (p.center, p.radius),
                                _ => return None,
                            };

                            let mid_p1 = Point::new(
                                (head_p0.x + tail_p0.x) * 0.5 + bendx,
                                (head_p0.y + tail_p0.y) * 0.5 + bendy,
                            );

                            let tail_versor = (tail_p0 - mid_p1) / (tail_p0 - mid_p1).hypot();
                            let head_versor = (head_p0 - mid_p1) / (head_p0 - mid_p1).hypot();

                            let mut marker_len = theme
                                .get_marker_by_name("arrowhead1")
                                .map(|m| m.get_width())
                                .unwrap_or(0.0);
                            if let Some(stroke) = theme.get_stroke_by_name("line-thin") {
                                marker_len += 2.0 * stroke.get_width();
                            }

                            let border_width = theme
                                .get_stroke_by_name("node")
                                .map(|s| s.get_width())
                                .unwrap_or(0.0);

                            let tail_p1 = tail_p0 - tail_versor * (tail_r + border_width);
                            let head_p1 =
                                head_p0 - head_versor * (head_r + marker_len + border_width);

                            return Some(BezPath::from_vec(vec![
                                PathEl::MoveTo(tail_p1),
                                PathEl::QuadTo(mid_p1, head_p1),
                            ]))
                        }
                    }
                }
            }
        }
        None
    }

    pub fn cubic_joint(
        &self,
        theme: &Theme,
        group_id: GroupId,
        tail: usize,
        head: usize,
        bend1x: f64,
        bend1y: f64,
        bend2x: f64,
        bend2y: f64,
    ) -> Option<BezPath> {
        if let Some(group) = self.get_group(group_id) {
            let items = group.get_crumb_items();
            if let Some(CrumbItem(tail_id, ..)) = items.get(tail) {
                if let Some(CrumbItem(head_id, ..)) = items.get(head) {
                    if let Some(tail_crumb) = self.get_crumb(*tail_id) {
                        if let Some(head_crumb) = self.get_crumb(*head_id) {
                            let (tail_p0, tail_r) = match tail_crumb {
                                Crumb::Circle(c) => (c.center, c.radius),
                                Crumb::Pin(p) => (p.center, p.radius),
                                _ => return None,
                            };
                            let (head_p0, head_r) = match head_crumb {
                                Crumb::Circle(c) => (c.center, c.radius),
                                Crumb::Pin(p) => (p.center, p.radius),
                                _ => return None,
                            };

                            let mid_p1 = Point::new(
                                (head_p0.x + tail_p0.x) * 0.5 + bend1x,
                                (head_p0.y + tail_p0.y) * 0.5 + bend1y,
                            );

                            let mid_p2 = Point::new(
                                (head_p0.x + tail_p0.x) * 0.5 + bend2x,
                                (head_p0.y + tail_p0.y) * 0.5 + bend2y,
                            );

                            let tail_versor = (tail_p0 - mid_p1) / (tail_p0 - mid_p1).hypot();
                            let head_versor = (head_p0 - mid_p2) / (head_p0 - mid_p2).hypot();

                            let mut marker_len = theme
                                .get_marker_by_name("arrowhead1")
                                .map(|m| m.get_width())
                                .unwrap_or(0.0);
                            if let Some(stroke) = theme.get_stroke_by_name("line-thin") {
                                marker_len += 2.0 * stroke.get_width();
                            }

                            let border_width = theme
                                .get_stroke_by_name("node")
                                .map(|s| s.get_width())
                                .unwrap_or(0.0);

                            let tail_p1 = tail_p0 - tail_versor * (tail_r + border_width);
                            let head_p1 =
                                head_p0 - head_versor * (head_r + marker_len + border_width);

                            return Some(BezPath::from_vec(vec![
                                PathEl::MoveTo(tail_p1),
                                PathEl::CurveTo(mid_p1, mid_p2, head_p1),
                            ]))
                        }
                    }
                }
            }
        }
        None
    }

    pub fn render<S, M, R>(
        &self,
        theme: &Theme,
        out_size: S,
        out_margin: M,
        rc: &mut R,
    ) -> Result<(), Box<dyn Error>>
    where
        S: Into<Size>,
        M: Into<Size>,
        R: RenderContext,
    {
        let out_size = out_size.into();
        let out_margin = out_margin.into();
        let out_scale = ((out_size.width - 2. * out_margin.width) / self.size.width)
            .min((out_size.height - 2. * out_margin.height) / self.size.height);

        let root_ts =
            TranslateScale::translate(out_margin.to_vec2()) * TranslateScale::scale(out_scale);

        rc.clear(None, theme.get_bg_color());

        for CrumbItem(crumb_id, ts, style_id) in self.all_crumbs(root_ts) {
            if let Some(crumb) = self.crumbs.get(crumb_id.0) {
                crumb.vis(rc, ts, style_id, theme);
            } else {
                // FIXME
                panic!()
            }
        }

        rc.finish()?;

        Ok(())
    }

    fn push_crumbs_of_a_group<'a>(
        &'a self,
        group: &'a Group,
        ts: TranslateScale,
        crumb_chain: &mut Vec<(CrumbList<'a>, TranslateScale)>,
    ) {
        crumb_chain.push((group.get_crumb_items().iter(), ts));

        for GroupItem(group_id, group_ts) in group.get_group_items().iter() {
            if let Some(group) = self.groups.get(group_id.0) {
                self.push_crumbs_of_a_group(group, ts * *group_ts, crumb_chain);
            } else {
                // FIXME
                panic!()
            }
        }
    }

    /// Collects all crumbs of a scene.
    ///
    /// Returns an iterator listing [`CrumbItem`]s containing their
    /// effective transformations computed in the depth-first
    /// traversal of the scene tree.
    pub fn all_crumbs(&self, root_ts: TranslateScale) -> CrumbChainIter {
        let mut crumb_chain = Vec::new();

        for group_id in self.roots.iter() {
            if let Some(group) = self.groups.get(group_id.0) {
                self.push_crumbs_of_a_group(group, root_ts, &mut crumb_chain);
            } else {
                // FIXME
                panic!()
            }
        }

        CrumbChainIter { crumb_chain }
    }

    pub fn simple_demo(theme: &Theme) -> Self {
        let mut scene = Scene::new((1000., 1000.));

        let border = scene.add_rect(Rect::new(0., 0., 1000., 1000.));
        let button = scene.add_rounded_rect(RoundedRect::new(250., 400., 450., 600., 10.));

        let lines = scene.add_grouped_lines(vec![
            (Line::new((0., 500.), (250., 0.)), theme.get("line-1")),
            (Line::new((0., 500.), (250., 1000.)), theme.get("line-1")),
            (Line::new((250., 1000.), (250., 0.)), theme.get("line-2")),
        ]);

        let rects = scene.add_group(Group::from_crumbs(vec![
            (button, theme.get("rect-1")),
            (button, theme.get("rect-2")),
        ]));

        let circle = scene.add_circle(Circle::new((133., 500.), 110.));

        let mixed_group = scene.add_group(
            Group::from_groups(vec![lines, rects]).with_crumb(circle, theme.get("circ-1")),
        );

        let triple_group = scene.add_group(
            Group::from_groups(vec![mixed_group])
                .with_group_item(GroupItem(
                    mixed_group,
                    0.5 * TranslateScale::translate((750., 0.).into()),
                ))
                .with_group_item(GroupItem(
                    mixed_group,
                    0.5 * TranslateScale::translate((750., 1000.).into()),
                )),
        );

        scene.add_root(
            Group::from_crumbs(vec![(border, theme.get("border"))])
                .with_group(triple_group)
                .with_group_item(GroupItem(
                    triple_group,
                    TranslateScale::translate((500., 0.).into()),
                )),
        );

        scene
    }
}

type CrumbList<'a> = slice::Iter<'a, CrumbItem>;

/// An iterator traversing all [`CrumbItem`]s of a [`Scene`].
///
/// Note: the effective [`TranslateScale`] transform of each
/// [`CrumbItem`] is composed on-the-fly, in the iterator's `next()`
/// method.
pub struct CrumbChainIter<'a> {
    crumb_chain: Vec<(CrumbList<'a>, TranslateScale)>,
}

impl<'a> Iterator for CrumbChainIter<'a> {
    type Item = CrumbItem;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((mut crumb_list, ts)) = self.crumb_chain.pop() {
            if let Some(item) = crumb_list.next() {
                self.crumb_chain.push((crumb_list, ts));

                return Some(CrumbItem(item.0, item.1 * ts, item.2))
            }
        }

        None
    }
}
