use std::{slice, io::Write, error::Error};
use piet_common::{
    RenderContext,
    kurbo::{Shape, Line, Rect, RoundedRect, TranslateScale, Size},
};
use crate::{Vis, Prim, PrimId, Group, GroupId, StyleId, Theme, WriteSvg, WriteSvgWithName};

#[derive(Clone, Default, Debug)]
pub struct Scene {
    size:   Size,
    prims:  Vec<Prim>,
    groups: Vec<Group>,
    roots:  Vec<GroupId>,
}

impl Scene {
    pub fn new<S: Into<Size>>(size: S) -> Self {
        Scene { size: size.into(), ..Default::default() }
    }

    pub fn add_line(&mut self, line: Line) -> PrimId {
        let id = self.prims.len();

        self.prims.push(Prim::Line(line));

        PrimId(id)
    }

    pub fn add_rect(&mut self, rect: Rect) -> PrimId {
        let id = self.prims.len();

        self.prims.push(Prim::Rect(rect));

        PrimId(id)
    }

    pub fn add_rounded_rect(&mut self, rect: RoundedRect) -> PrimId {
        let id = self.prims.len();

        self.prims.push(Prim::RoundedRect(rect));

        PrimId(id)
    }

    pub fn add_prim(&mut self, prim: Prim) -> PrimId {
        let id = self.prims.len();

        self.prims.push(prim);

        PrimId(id)
    }

    pub fn add_group(&mut self, group: Group) -> GroupId {
        let id = self.groups.len();

        self.groups.push(group);

        GroupId(id)
    }

    pub fn add_grouped_prims<I>(&mut self, prims: I) -> GroupId
    where
        I: IntoIterator<Item = (Prim, Option<StyleId>)>,
    {
        let prims = prims.into_iter().map(|(p, s)| (self.add_prim(p), s));
        let group = Group::from_prims(prims);

        self.add_group(group)
    }

    pub fn add_grouped_lines<I>(&mut self, lines: I) -> GroupId
    where
        I: IntoIterator<Item = (Line, Option<StyleId>)>,
    {
        let prims = lines.into_iter().map(|(l, s)| (self.add_line(l), s));
        let group = Group::from_prims(prims);

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

        for (prim_id, style_id, ts) in self.all_prims(root_ts) {
            if let Some(prim) = self.prims.get(prim_id.0) {
                match *prim {
                    Prim::Line(line) => (ts * line).vis(style_id, theme, rc),
                    Prim::Rect(rect) => (ts * rect).vis(style_id, theme, rc),
                    Prim::RoundedRect(rect) => (ts * rect).vis(style_id, theme, rc),
                }
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
        writeln!(&mut svg, "  <rect width=\"100%\" height=\"100%\" ")?;
        bg_color.write_svg_with_name(&mut svg, "fill")?;
        writeln!(&mut svg, " />")?;

        for (prim_id, style_id, ts) in self.all_prims(root_ts) {
            if let Some(prim) = self.prims.get(prim_id.0) {
                // FIXME prim.write_to(&mut svg)?;
                match *prim {
                    Prim::Line(line) => {
                        // FIXME use `<line>`
                        let path = (ts * line).into_bez_path(1e-3);

                        write!(&mut svg, "  <path d=\"")?;
                        path.write_to(&mut svg)?;
                        write!(&mut svg, "\" ")?;

                        if let Some(stroke) =
                            theme.get_stroke(style_id).or_else(|| theme.get_default_stroke())
                        {
                            stroke.write_svg(&mut svg)?;
                        }

                        writeln!(&mut svg, "/>")?;
                    }
                    Prim::Rect(rect) => {
                        // FIXME use `<rect>`
                        let path = (ts * rect).into_bez_path(1e-3);

                        write!(&mut svg, "  <path d=\"")?;
                        path.write_to(&mut svg)?;
                        write!(&mut svg, "\" ")?;

                        if let Some(style) = theme.get_style(style_id) {
                            style.write_svg(&mut svg)?;
                        }

                        writeln!(&mut svg, "/>")?;
                    }
                    Prim::RoundedRect(rect) => {
                        // FIXME use `<rect rx>`
                        let path = (ts * rect).into_bez_path(1e-3);

                        write!(&mut svg, "  <path d=\"")?;
                        path.write_to(&mut svg)?;
                        write!(&mut svg, "\" ")?;

                        if let Some(style) = theme.get_style(style_id) {
                            style.write_svg(&mut svg)?;
                        }

                        writeln!(&mut svg, "/>")?;
                    }
                }
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

    fn push_prims_of_a_group<'a>(
        &'a self,
        group: &'a Group,
        ts: TranslateScale,
        prim_chain: &mut Vec<(PrimList<'a>, TranslateScale)>,
    ) {
        prim_chain.push((group.get_prims().iter(), ts));

        for (group_id, group_ts) in group.get_groups().iter() {
            if let Some(group) = self.groups.get(group_id.0) {
                self.push_prims_of_a_group(group, ts * *group_ts, prim_chain);
            } else {
                // FIXME
                panic!()
            }
        }
    }

    /// Deep traversal with transformations applied.
    pub fn all_prims(&self, root_ts: TranslateScale) -> PrimChainIter {
        let mut prim_chain = Vec::new();

        for group_id in self.roots.iter() {
            if let Some(group) = self.groups.get(group_id.0) {
                self.push_prims_of_a_group(group, root_ts, &mut prim_chain);
            } else {
                // FIXME
                panic!()
            }
        }

        PrimChainIter { prim_chain }
    }
}

type PrimItem = (PrimId, Option<StyleId>, TranslateScale);
type PrimList<'a> = slice::Iter<'a, PrimItem>;

pub struct PrimChainIter<'a> {
    prim_chain: Vec<(PrimList<'a>, TranslateScale)>,
}

impl<'a> Iterator for PrimChainIter<'a> {
    type Item = PrimItem;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((mut prim_list, ts)) = self.prim_chain.pop() {
            if let Some(item) = prim_list.next() {
                self.prim_chain.push((prim_list, ts));

                return Some((item.0, item.1, item.2 * ts))
            }
        }

        None
    }
}
