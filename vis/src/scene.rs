use std::{slice, io::Write, error::Error};
use piet::RenderContext;
use kurbo::{Line, Rect, RoundedRect, Circle, TranslateScale, Size};
use crate::{
    Vis, WriteSvgWithStyle, WriteSvgWithName, Crumb, CrumbId, CrumbItem, Group, GroupId, GroupItem,
    StyleId, Theme,
};

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
        let group = Group::from_crumb_ids(crumbs);

        self.add_group(group)
    }

    pub fn add_grouped_lines<I>(&mut self, lines: I) -> GroupId
    where
        I: IntoIterator<Item = (Line, Option<StyleId>)>,
    {
        let crumbs = lines.into_iter().map(|(l, s)| (self.add_line(l), s));
        let group = Group::from_crumb_ids(crumbs);

        self.add_group(group)
    }

    pub fn add_root(&mut self, group: Group) -> GroupId {
        let group_id = GroupId(self.groups.len());

        self.groups.push(group);
        self.roots.push(group_id);

        group_id
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

        rc.clear(theme.get_bg_color());

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

    pub fn to_svg<S, M>(
        &self,
        theme: &Theme,
        out_size: S,
        out_margin: M,
    ) -> Result<String, Box<dyn Error>>
    where
        S: Into<Size>,
        M: Into<Size>,
    {
        let out_size = out_size.into();
        let out_margin = out_margin.into();
        let out_scale = ((out_size.width - 2. * out_margin.width) / self.size.width)
            .min((out_size.height - 2. * out_margin.height) / self.size.height);
        let root_ts =
            TranslateScale::translate(out_margin.to_vec2()) * TranslateScale::scale(out_scale);

        let mut svg = Vec::new();

        writeln!(&mut svg, "<!DOCTYPE html>")?;
        writeln!(&mut svg, "<html>")?;
        writeln!(&mut svg, "<body>")?;
        writeln!(
            &mut svg,
            "<svg width=\"{}\" height=\"{}\">",
            out_size.width.round(),
            out_size.height.round()
        )?;
        writeln!(&mut svg, "  <defs>")?;
        for (name, spec) in theme.get_named_gradspecs() {
            spec.write_svg_with_name(&mut svg, name)?;
        }
        writeln!(&mut svg, "  </defs>")?;

        let bg_color = theme.get_bg_color();
        write!(&mut svg, "  <rect width=\"100%\" height=\"100%\" ")?;
        bg_color.write_svg_with_name(&mut svg, "fill")?;
        writeln!(&mut svg, " />")?;

        for CrumbItem(crumb_id, ts, style_id) in self.all_crumbs(root_ts) {
            if let Some(crumb) = self.crumbs.get(crumb_id.0) {
                crumb.write_svg_with_style(&mut svg, ts, style_id, theme)?;
            } else {
                // FIXME
                panic!()
            }
        }

        writeln!(&mut svg, "</svg>")?;
        writeln!(&mut svg, "</body>")?;
        writeln!(&mut svg, "</html>")?;

        let svg = String::from_utf8(svg)?;

        Ok(svg)
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

        let rects = scene.add_group(Group::from_crumb_ids(vec![
            (button, theme.get("rect-1")),
            (button, theme.get("rect-2")),
        ]));

        let circle = scene.add_circle(Circle::new((133., 500.), 110.));

        let mixed_group = scene.add_group(
            Group::from_group_ids(vec![lines, rects]).with_crumb_id(circle, theme.get("circ-1")),
        );

        let triple_group = scene.add_group(
            Group::from_group_ids(vec![mixed_group])
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
            Group::from_crumb_ids(vec![(border, theme.get("border"))])
                .with_group_id(triple_group)
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
/// [`CrumbItem`] is composed on-the-fly, in the iterator's [`next()`]
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
