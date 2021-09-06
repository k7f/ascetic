use kurbo::{Point, Line, Arc, BezPath, PathEl};
use crate::{Scene, Theme, StyleId, Group, GroupId, Crumb, CrumbItem};

enum Joints {
    Lines(Vec<(usize, usize)>),
    Polylines(Vec<(usize, usize, Vec<(f64, f64)>)>),
    Arcs(Vec<(usize, usize, f64)>),
    Curves(Vec<(usize, usize, Vec<(f64, f64)>)>),
}

pub struct JointBuilder {
    tail_group: GroupId,
    head_group: GroupId,
    joints:     Vec<(Option<StyleId>, Joints)>,
}

impl JointBuilder {
    fn new(tail_group: GroupId, head_group: GroupId) -> Self {
        let joints = Vec::new();

        JointBuilder { tail_group, head_group, joints }
    }

    fn build(self, scene: &mut Scene, theme: &Theme) -> GroupId {
        let tail_group = self.tail_group;
        let head_group = self.head_group;
        let mut joints_group = Group::default();

        for (style_id, joints) in self.joints {
            match joints {
                Joints::Lines(lines) => {
                    joints_group.add_crumbs(lines.iter().map(|(tail, head)| {
                        (
                            scene.add_line(
                                scene
                                    .line_joint(
                                        style_id,
                                        theme,
                                        (tail_group, *tail),
                                        (head_group, *head),
                                    )
                                    .unwrap(),
                            ),
                            style_id,
                        )
                    }))
                }
                Joints::Polylines(polylines) => {
                    joints_group.add_crumbs(polylines.iter().map(|(tail, head, pull)| {
                        (
                            scene.add_crumb(Crumb::Path(
                                scene
                                    .polyline_joint(
                                        style_id,
                                        theme,
                                        (tail_group, *tail),
                                        (head_group, *head),
                                        pull.as_slice(),
                                    )
                                    .unwrap(),
                            )),
                            style_id,
                        )
                    }))
                }
                Joints::Arcs(arcs) => {
                    joints_group.add_crumbs(arcs.iter().map(|(tail, head, radius)| {
                        (
                            scene.add_arc(
                                scene
                                    .arc_joint(
                                        style_id,
                                        theme,
                                        (tail_group, *tail),
                                        (head_group, *head),
                                        *radius,
                                    )
                                    .unwrap(),
                            ),
                            style_id,
                        )
                    }))
                }
                Joints::Curves(curves) => {
                    joints_group.add_crumbs(curves.iter().map(|(tail, head, pull)| {
                        (
                            scene.add_crumb(
                                if let Some(((pull1x, pull1y), rest)) = pull.split_first() {
                                    if let Some((pull2x, pull2y)) = rest.last() {
                                        Crumb::Path(
                                            scene
                                                .cubic_joint(
                                                    style_id,
                                                    theme,
                                                    (tail_group, *tail),
                                                    (head_group, *head),
                                                    *pull1x,
                                                    *pull1y,
                                                    *pull2x,
                                                    *pull2y,
                                                )
                                                .unwrap(),
                                        )
                                    } else {
                                        Crumb::Path(
                                            scene
                                                .quad_joint(
                                                    style_id,
                                                    theme,
                                                    (tail_group, *tail),
                                                    (head_group, *head),
                                                    *pull1x,
                                                    *pull1y,
                                                )
                                                .unwrap(),
                                        )
                                    }
                                } else {
                                    Crumb::Line(
                                        scene
                                            .line_joint(
                                                style_id,
                                                theme,
                                                (tail_group, *tail),
                                                (head_group, *head),
                                            )
                                            .unwrap(),
                                    )
                                },
                            ),
                            style_id,
                        )
                    }))
                }
            }
        }

        scene.add_group(joints_group)
    }
}

pub trait Joint {
    fn with_lines<I>(self, style_id: Option<StyleId>, lines: I) -> Self
    where
        I: IntoIterator<Item = (usize, usize)>;

    fn with_polylines<I, J>(self, style_id: Option<StyleId>, polylines: I) -> Self
    where
        I: IntoIterator<Item = (usize, usize, J)>,
        J: IntoIterator<Item = (f64, f64)>;

    fn with_arcs<I>(self, style_id: Option<StyleId>, arcs: I) -> Self
    where
        I: IntoIterator<Item = (usize, usize, f64)>;

    fn with_curves<I, J>(self, style_id: Option<StyleId>, curves: I) -> Self
    where
        I: IntoIterator<Item = (usize, usize, J)>,
        J: IntoIterator<Item = (f64, f64)>;

    fn as_group(self, theme: &Theme) -> GroupId;
}

impl Joint for (&mut Scene, JointBuilder) {
    fn with_lines<I>(mut self, style_id: Option<StyleId>, lines: I) -> Self
    where
        I: IntoIterator<Item = (usize, usize)>,
    {
        self.1.joints.push((style_id, Joints::Lines(lines.into_iter().collect())));
        self
    }

    fn with_polylines<I, J>(mut self, style_id: Option<StyleId>, polylines: I) -> Self
    where
        I: IntoIterator<Item = (usize, usize, J)>,
        J: IntoIterator<Item = (f64, f64)>,
    {
        self.1.joints.push((
            style_id,
            Joints::Polylines(
                polylines.into_iter().map(|(t, h, b)| (t, h, b.into_iter().collect())).collect(),
            ),
        ));
        self
    }

    fn with_arcs<I>(mut self, style_id: Option<StyleId>, arcs: I) -> Self
    where
        I: IntoIterator<Item = (usize, usize, f64)>,
    {
        self.1.joints.push((style_id, Joints::Arcs(arcs.into_iter().collect())));
        self
    }

    fn with_curves<I, J>(mut self, style_id: Option<StyleId>, curves: I) -> Self
    where
        I: IntoIterator<Item = (usize, usize, J)>,
        J: IntoIterator<Item = (f64, f64)>,
    {
        self.1.joints.push((
            style_id,
            Joints::Curves(
                curves.into_iter().map(|(t, h, b)| (t, h, b.into_iter().collect())).collect(),
            ),
        ));
        self
    }

    #[inline]
    fn as_group(self, theme: &Theme) -> GroupId {
        self.1.build(self.0, theme)
    }
}

impl Scene {
    pub fn join(&mut self, tail_group: GroupId, head_group: GroupId) -> (&mut Self, JointBuilder) {
        (self, JointBuilder::new(tail_group, head_group))
    }

    pub fn line_joint(
        &self,
        style_id: Option<StyleId>,
        theme: &Theme,
        tail: (GroupId, usize),
        head: (GroupId, usize),
    ) -> Option<Line> {
        if let Some(tail_group) = self.get_group(tail.0) {
            if let Some(head_group) = self.get_group(head.0) {
                let tail_items = tail_group.get_crumb_items();
                let head_items = head_group.get_crumb_items();
                if let Some(CrumbItem(tail_id, _tail_ts, tail_style_id)) = tail_items.get(tail.1) {
                    if let Some(CrumbItem(head_id, _head_ts, head_style_id)) =
                        head_items.get(head.1)
                    {
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

                                let tail_border_width = theme
                                    .get_stroke(*tail_style_id)
                                    .map(|s| s.get_width())
                                    .unwrap_or(0.0);
                                let head_border_width = theme
                                    .get_stroke(*head_style_id)
                                    .map(|s| s.get_width())
                                    .unwrap_or(0.0);
                                let (tail_marker_len, head_marker_len) =
                                    theme.get_marker_width(style_id);

                                let tail_p1 = tail_p0
                                    + versor * (tail_r + 0.5 * tail_border_width + tail_marker_len);
                                let head_p1 = head_p0
                                    - versor * (head_r + 0.5 * head_border_width + head_marker_len);

                                return Some(Line::new(tail_p1, head_p1))
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub fn polyline_joint(
        &self,
        style_id: Option<StyleId>,
        theme: &Theme,
        tail: (GroupId, usize),
        head: (GroupId, usize),
        pull: &[(f64, f64)],
    ) -> Option<BezPath> {
        if let Some(tail_group) = self.get_group(tail.0) {
            if let Some(head_group) = self.get_group(head.0) {
                let tail_items = tail_group.get_crumb_items();
                let head_items = head_group.get_crumb_items();
                if let Some(CrumbItem(tail_id, _tail_ts, tail_style_id)) = tail_items.get(tail.1) {
                    if let Some(CrumbItem(head_id, _head_ts, head_style_id)) =
                        head_items.get(head.1)
                    {
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

                                let tail_border_width = theme
                                    .get_stroke(*tail_style_id)
                                    .map(|s| s.get_width())
                                    .unwrap_or(0.0);
                                let head_border_width = theme
                                    .get_stroke(*head_style_id)
                                    .map(|s| s.get_width())
                                    .unwrap_or(0.0);
                                let (tail_marker_len, head_marker_len) =
                                    theme.get_marker_width(style_id);

                                if let Some(((pull1x, pull1y), rest)) = pull.split_first() {
                                    let mid_p1 = Point::new(
                                        (head_p0.x + tail_p0.x) * 0.5 + pull1x,
                                        (head_p0.y + tail_p0.y) * 0.5 + pull1y,
                                    );

                                    if let Some((pull2x, pull2y)) = rest.last() {
                                        let mid_p2 = Point::new(
                                            (head_p0.x + tail_p0.x) * 0.5 + pull2x,
                                            (head_p0.y + tail_p0.y) * 0.5 + pull2y,
                                        );
                                        let tail_versor =
                                            (tail_p0 - mid_p1) / (tail_p0 - mid_p1).hypot();
                                        let head_versor =
                                            (head_p0 - mid_p2) / (head_p0 - mid_p2).hypot();
                                        let tail_p1 = tail_p0
                                            - tail_versor
                                                * (tail_r
                                                    + 0.5 * tail_border_width
                                                    + tail_marker_len);
                                        let head_p1 = head_p0
                                            - head_versor
                                                * (head_r
                                                    + 0.5 * head_border_width
                                                    + head_marker_len);

                                        return Some(BezPath::from_vec(vec![
                                            PathEl::MoveTo(tail_p1),
                                            PathEl::LineTo(mid_p1),
                                            PathEl::LineTo(mid_p2),
                                            PathEl::LineTo(head_p1),
                                        ]))
                                    } else {
                                        let tail_versor =
                                            (tail_p0 - mid_p1) / (tail_p0 - mid_p1).hypot();
                                        let head_versor =
                                            (head_p0 - mid_p1) / (head_p0 - mid_p1).hypot();
                                        let tail_p1 = tail_p0
                                            - tail_versor
                                                * (tail_r
                                                    + 0.5 * tail_border_width
                                                    + tail_marker_len);
                                        let head_p1 = head_p0
                                            - head_versor
                                                * (head_r
                                                    + 0.5 * head_border_width
                                                    + head_marker_len);

                                        return Some(BezPath::from_vec(vec![
                                            PathEl::MoveTo(tail_p1),
                                            PathEl::LineTo(mid_p1),
                                            PathEl::LineTo(head_p1),
                                        ]))
                                    }
                                } else {
                                    let versor = (head_p0 - tail_p0) / (head_p0 - tail_p0).hypot();
                                    let tail_p1 = tail_p0
                                        + versor
                                            * (tail_r + 0.5 * tail_border_width + tail_marker_len);
                                    let head_p1 = head_p0
                                        - versor
                                            * (head_r + 0.5 * head_border_width + head_marker_len);

                                    return Some(BezPath::from_vec(vec![
                                        PathEl::MoveTo(tail_p1),
                                        PathEl::LineTo(head_p1),
                                    ]))
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub fn arc_joint(
        &self,
        style_id: Option<StyleId>,
        theme: &Theme,
        tail: (GroupId, usize),
        head: (GroupId, usize),
        mut radius: f64,
    ) -> Option<Arc> {
        if let Some(tail_group) = self.get_group(tail.0) {
            if let Some(head_group) = self.get_group(head.0) {
                let tail_items = tail_group.get_crumb_items();
                let head_items = head_group.get_crumb_items();
                if let Some(CrumbItem(tail_id, _tail_ts, tail_style_id)) = tail_items.get(tail.1) {
                    if let Some(CrumbItem(head_id, _head_ts, head_style_id)) =
                        head_items.get(head.1)
                    {
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

                                let tail_border_width = theme
                                    .get_stroke(*tail_style_id)
                                    .map(|s| s.get_width())
                                    .unwrap_or(0.0);
                                let head_border_width = theme
                                    .get_stroke(*head_style_id)
                                    .map(|s| s.get_width())
                                    .unwrap_or(0.0);
                                let (tail_marker_len, head_marker_len) =
                                    theme.get_marker_width(style_id);

                                let tail_apex_angle = 2.0
                                    * ((tail_r + 0.5 * tail_border_width + tail_marker_len)
                                        / (2.0 * radius))
                                        .asin();
                                let head_apex_angle = 2.0
                                    * ((head_r + 0.5 * head_border_width + head_marker_len)
                                        / (2.0 * radius))
                                        .asin();
                                let total_apex_angle = tail_apex_angle + head_apex_angle;

                                let start_angle =
                                    (tail_p0.y - center.y).atan2(tail_p0.x - center.x);
                                let mut sweep_angle = (head_p0.y - center.y)
                                    .atan2(head_p0.x - center.x)
                                    - start_angle;

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
        }
        None
    }

    pub fn quad_joint(
        &self,
        style_id: Option<StyleId>,
        theme: &Theme,
        tail: (GroupId, usize),
        head: (GroupId, usize),
        pullx: f64,
        pully: f64,
    ) -> Option<BezPath> {
        if let Some(tail_group) = self.get_group(tail.0) {
            if let Some(head_group) = self.get_group(head.0) {
                let tail_items = tail_group.get_crumb_items();
                let head_items = head_group.get_crumb_items();
                if let Some(CrumbItem(tail_id, _tail_ts, tail_style_id)) = tail_items.get(tail.1) {
                    if let Some(CrumbItem(head_id, _head_ts, head_style_id)) =
                        head_items.get(head.1)
                    {
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
                                    (head_p0.x + tail_p0.x) * 0.5 + pullx,
                                    (head_p0.y + tail_p0.y) * 0.5 + pully,
                                );

                                let tail_versor = (tail_p0 - mid_p1) / (tail_p0 - mid_p1).hypot();
                                let head_versor = (head_p0 - mid_p1) / (head_p0 - mid_p1).hypot();

                                let tail_border_width = theme
                                    .get_stroke(*tail_style_id)
                                    .map(|s| s.get_width())
                                    .unwrap_or(0.0);
                                let head_border_width = theme
                                    .get_stroke(*head_style_id)
                                    .map(|s| s.get_width())
                                    .unwrap_or(0.0);
                                let (tail_marker_len, head_marker_len) =
                                    theme.get_marker_width(style_id);

                                let tail_p1 = tail_p0
                                    - tail_versor
                                        * (tail_r + 0.5 * tail_border_width + tail_marker_len);
                                let head_p1 = head_p0
                                    - head_versor
                                        * (head_r + 0.5 * head_border_width + head_marker_len);

                                return Some(BezPath::from_vec(vec![
                                    PathEl::MoveTo(tail_p1),
                                    PathEl::QuadTo(mid_p1, head_p1),
                                ]))
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub fn cubic_joint(
        &self,
        style_id: Option<StyleId>,
        theme: &Theme,
        tail: (GroupId, usize),
        head: (GroupId, usize),
        pull1x: f64,
        pull1y: f64,
        pull2x: f64,
        pull2y: f64,
    ) -> Option<BezPath> {
        if let Some(tail_group) = self.get_group(tail.0) {
            if let Some(head_group) = self.get_group(head.0) {
                let tail_items = tail_group.get_crumb_items();
                let head_items = head_group.get_crumb_items();
                if let Some(CrumbItem(tail_id, _tail_ts, tail_style_id)) = tail_items.get(tail.1) {
                    if let Some(CrumbItem(head_id, _head_ts, head_style_id)) =
                        head_items.get(head.1)
                    {
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
                                    (head_p0.x + tail_p0.x) * 0.5 + pull1x,
                                    (head_p0.y + tail_p0.y) * 0.5 + pull1y,
                                );

                                let mid_p2 = Point::new(
                                    (head_p0.x + tail_p0.x) * 0.5 + pull2x,
                                    (head_p0.y + tail_p0.y) * 0.5 + pull2y,
                                );

                                let tail_versor = (tail_p0 - mid_p1) / (tail_p0 - mid_p1).hypot();
                                let head_versor = (head_p0 - mid_p2) / (head_p0 - mid_p2).hypot();

                                let tail_border_width = theme
                                    .get_stroke(*tail_style_id)
                                    .map(|s| s.get_width())
                                    .unwrap_or(0.0);
                                let head_border_width = theme
                                    .get_stroke(*head_style_id)
                                    .map(|s| s.get_width())
                                    .unwrap_or(0.0);
                                let (tail_marker_len, head_marker_len) =
                                    theme.get_marker_width(style_id);

                                let tail_p1 = tail_p0
                                    - tail_versor
                                        * (tail_r + 0.5 * tail_border_width + tail_marker_len);
                                let head_p1 = head_p0
                                    - head_versor
                                        * (head_r + 0.5 * head_border_width + head_marker_len);

                                return Some(BezPath::from_vec(vec![
                                    PathEl::MoveTo(tail_p1),
                                    PathEl::CurveTo(mid_p1, mid_p2, head_p1),
                                ]))
                            }
                        }
                    }
                }
            }
        }
        None
    }
}
