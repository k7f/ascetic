use kurbo::{Line, Rect, RoundedRect, Circle, Arc, TranslateScale, Size};
use crate::{Crumb, CrumbId, CrumbItem, Group, GroupId, GroupItem, StyleId, Theme, VisError};

#[derive(Clone, Debug)]
struct Layer {
    group_id:   GroupId,
    is_visible: bool,
    z_index:    i64,
}

impl Layer {
    fn new(group_id: GroupId) -> Self {
        Layer { group_id, is_visible: true, z_index: 0 }
    }
}

#[derive(Clone, Default, Debug)]
pub struct Scene {
    size:   Size,
    crumbs: Vec<Crumb>,
    groups: Vec<Group>,
    layers: Vec<Layer>,
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
    pub fn get_crumb_mut(&mut self, crumb_id: CrumbId) -> Option<&mut Crumb> {
        self.crumbs.get_mut(crumb_id.0)
    }

    #[inline]
    pub fn get_group(&self, group_id: GroupId) -> Option<&Group> {
        self.groups.get(group_id.0)
    }

    #[inline]
    pub fn get_group_mut(&mut self, group_id: GroupId) -> Option<&mut Group> {
        self.groups.get_mut(group_id.0)
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

    pub fn add_arc(&mut self, arc: Arc) -> CrumbId {
        let id = self.crumbs.len();

        self.crumbs.push(Crumb::Arc(arc));

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
        let crumbs = crumbs.into_iter().map(|(c, s)| (self.add_crumb(c), s));
        let group = Group::from_crumbs(crumbs);

        self.add_group(group)
    }

    pub fn add_named_crumbs<S, I>(&mut self, name: S, crumbs: I) -> GroupId
    where
        S: AsRef<str>,
        I: IntoIterator<Item = (Crumb, Option<StyleId>)>,
    {
        let crumbs = crumbs.into_iter().map(|(c, s)| (self.add_crumb(c), s));
        let group = Group::from_crumbs(crumbs).with_name(name);

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

    pub fn add_layer(&mut self, group: Group) -> GroupId {
        let group_id = GroupId(self.groups.len());
        let layer = Layer::new(group_id);

        self.groups.push(group);
        self.layers.push(layer);

        group_id
    }

    pub fn add_layer_by_id(&mut self, group_id: GroupId) -> Result<(), VisError> {
        if self.all_groups()?.any(|(_, id)| id == group_id) {
            Err(VisError::group_reuse_attempt(group_id))
        } else {
            let layer = Layer::new(group_id);

            self.layers.push(layer);
            Ok(())
        }
    }

    pub fn set_z_index(&mut self, group_id: GroupId, z_index: i64) -> Result<(), VisError> {
        if let Some(layer) = self.layers.iter_mut().find(|layer| layer.group_id == group_id) {
            layer.z_index = z_index;
            Ok(())
        } else {
            Err(VisError::layer_missing_for_id(group_id))
        }
    }

    pub fn hide_layer(&mut self, group_id: GroupId) -> Result<(), VisError> {
        if let Some(layer) = self.layers.iter_mut().find(|layer| layer.group_id == group_id) {
            layer.is_visible = false;
            Ok(())
        } else {
            Err(VisError::layer_missing_for_id(group_id))
        }
    }

    pub fn show_layer(&mut self, group_id: GroupId) -> Result<(), VisError> {
        if let Some(layer) = self.layers.iter_mut().find(|layer| layer.group_id == group_id) {
            layer.is_visible = true;
            Ok(())
        } else {
            Err(VisError::layer_missing_for_id(group_id))
        }
    }

    /// Lists all top-level groups of the [`Scene`] in reversed
    /// stacking order (top-down).
    pub fn get_layers(&self) -> Vec<GroupId> {
        let mut z_stack = self.layers.clone();

        z_stack.sort_by_key(|layer| -layer.z_index);

        z_stack.iter().map(|layer| layer.group_id).collect()
    }

    /// Lists visible top-level groups of the [`Scene`] in reversed
    /// stacking order (top-down).
    pub fn get_visible_layers(&self) -> Vec<GroupId> {
        let mut z_stack: Vec<_> = self.layers.iter().filter(|layer| layer.is_visible).collect();

        z_stack.sort_by_key(|layer| -layer.z_index);

        z_stack.iter().map(|layer| layer.group_id).collect()
    }

    fn push_crumbs_of_a_group<'a>(
        &'a self,
        group: &'a Group,
        mut level: usize,
        ts: TranslateScale,
        crumb_chain: &mut Vec<(usize, CrumbList<'a>, TranslateScale)>,
    ) -> Result<(), VisError> {
        level += 1;
        crumb_chain.push((level, group.get_crumb_items().iter(), ts));

        for GroupItem(group_id, group_ts) in group.get_group_items().iter() {
            if let Some(group) = self.groups.get(group_id.0) {
                self.push_crumbs_of_a_group(group, level, ts * *group_ts, crumb_chain)?;
            } else {
                return Err(VisError::group_missing_for_id(*group_id))
            }
        }
        Ok(())
    }

    fn traverse_crumbs(
        &self,
        z_stack: Vec<GroupId>,
        root_ts: TranslateScale,
    ) -> Result<CrumbChainIter, VisError> {
        let mut crumb_chain = Vec::new();

        for group_id in z_stack.iter() {
            if let Some(group) = self.groups.get(group_id.0) {
                self.push_crumbs_of_a_group(group, 0, root_ts, &mut crumb_chain)?;
            } else {
                return Err(VisError::group_missing_for_id(*group_id))
            }
        }

        Ok(CrumbChainIter { crumb_chain })
    }

    /// Collects all crumbs of a scene.
    ///
    /// Returns an iterator listing [`CrumbItem`]s containing their
    /// effective transformations computed in the depth-first
    /// post-order traversal of the scene tree.  Stacking order of
    /// layers is respected.
    #[inline]
    pub fn all_crumbs(&self, root_ts: TranslateScale) -> Result<CrumbChainIter, VisError> {
        self.traverse_crumbs(self.get_layers(), root_ts)
    }

    /// Collects all visible crumbs of a scene.
    ///
    /// Returns an iterator listing [`CrumbItem`]s containing their
    /// effective transformations computed in the depth-first
    /// post-order traversal of the scene tree.  Stacking order of
    /// layers is respected.
    #[inline]
    pub fn all_visible_crumbs(&self, root_ts: TranslateScale) -> Result<CrumbChainIter, VisError> {
        self.traverse_crumbs(self.get_visible_layers(), root_ts)
    }

    fn push_subgroups_of_a_group<'a>(
        &'a self,
        group: &'a Group,
        mut level: usize,
        group_chain: &mut Vec<(usize, GroupList<'a>)>,
    ) -> Result<(), VisError> {
        level += 1;
        group_chain.push((level, GroupList::Items(group.get_group_items().iter())));

        for GroupItem(group_id, ..) in group.get_group_items().iter() {
            if let Some(group) = self.groups.get(group_id.0) {
                self.push_subgroups_of_a_group(group, level, group_chain)?;
            } else {
                return Err(VisError::group_missing_for_id(*group_id))
            }
        }
        Ok(())
    }

    /// Collects all groups of a scene.
    ///
    /// Returns an iterator listing [`GroupId`]s in the bottom-up
    /// level order traversal of the scene tree.  Stacking order of
    /// layers is ignored.
    pub fn all_groups(&self) -> Result<GroupChainIter, VisError> {
        let mut group_chain = vec![(0, GroupList::Layers(self.layers.iter()))];

        for Layer { group_id, .. } in self.layers.iter() {
            if let Some(group) = self.groups.get(group_id.0) {
                self.push_subgroups_of_a_group(group, 0, &mut group_chain)?;
            } else {
                return Err(VisError::group_missing_for_id(*group_id))
            }
        }

        Ok(GroupChainIter { group_chain })
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

        scene.add_layer(
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

type CrumbList<'a> = std::slice::Iter<'a, CrumbItem>;

/// An iterator traversing all [`CrumbItem`]s of a [`Scene`].
///
/// Note: the effective [`TranslateScale`] transform of each
/// [`CrumbItem`] is composed on-the-fly, in the iterator's `next()`
/// method.
pub struct CrumbChainIter<'a> {
    crumb_chain: Vec<(usize, CrumbList<'a>, TranslateScale)>,
}

impl<'a> Iterator for CrumbChainIter<'a> {
    type Item = (usize, CrumbItem);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((level, mut crumb_list, ts)) = self.crumb_chain.pop() {
            if let Some(item) = crumb_list.next() {
                self.crumb_chain.push((level, crumb_list, ts));

                return Some((level, CrumbItem(item.0, item.1 * ts, item.2)))
            }
        }

        None
    }
}

enum GroupList<'a> {
    Layers(std::slice::Iter<'a, Layer>),
    Items(std::slice::Iter<'a, GroupItem>),
}

/// An iterator traversing all [`GroupId`]s referenced in a [`Scene`].
pub struct GroupChainIter<'a> {
    group_chain: Vec<(usize, GroupList<'a>)>,
}

impl<'a> Iterator for GroupChainIter<'a> {
    type Item = (usize, GroupId);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((level, mut group_list)) = self.group_chain.pop() {
            if let Some(group_id) = match group_list {
                GroupList::Layers(ref mut layers) => {
                    if let Some(Layer { group_id, .. }) = layers.next() {
                        Some(group_id)
                    } else {
                        None
                    }
                }
                GroupList::Items(ref mut items) => {
                    if let Some(GroupItem(group_id, ..)) = items.next() {
                        Some(group_id)
                    } else {
                        None
                    }
                }
            } {
                self.group_chain.push((level, group_list));

                return Some((level, *group_id))
            }
        }

        None
    }
}
