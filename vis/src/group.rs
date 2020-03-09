use kurbo::TranslateScale;
use crate::{CrumbId, CrumbItem, StyleId};

const IDENTITY: TranslateScale = TranslateScale::scale(1.0);

#[derive(Clone, Copy, Debug)]
pub struct GroupId(pub usize);

#[derive(Clone, Copy, Debug)]
pub struct GroupItem(pub GroupId, pub TranslateScale);

#[derive(Clone, Debug)]
pub struct Group {
    crumbs: Vec<CrumbItem>,
    groups: Vec<GroupItem>,
}

impl Group {
    pub fn from_crumb_items<I>(crumbs: I) -> Self
    where
        I: IntoIterator<Item = CrumbItem>,
    {
        let crumbs = crumbs.into_iter().collect();
        let groups = Vec::new();

        Group { crumbs, groups }
    }

    pub fn from_group_items<I>(groups: I) -> Self
    where
        I: IntoIterator<Item = GroupItem>,
    {
        let crumbs = Vec::new();
        let groups = groups.into_iter().collect();

        Group { crumbs, groups }
    }

    pub fn from_crumb_ids<I>(crumbs: I) -> Self
    where
        I: IntoIterator<Item = (CrumbId, Option<StyleId>)>,
    {
        let crumbs = crumbs.into_iter().map(|(p, s)| CrumbItem(p, IDENTITY, s)).collect();
        let groups = Vec::new();

        Group { crumbs, groups }
    }

    pub fn from_group_ids<I>(groups: I) -> Self
    where
        I: IntoIterator<Item = GroupId>,
    {
        let crumbs = Vec::new();
        let groups = groups.into_iter().map(|g| GroupItem(g, IDENTITY)).collect();

        Group { crumbs, groups }
    }

    pub fn with_crumb_item(mut self, crumb: CrumbItem) -> Self {
        self.crumbs.push(crumb);
        self
    }

    pub fn with_group_item(mut self, group: GroupItem) -> Self {
        self.groups.push(group);
        self
    }

    pub fn with_crumb_id(mut self, crumb_id: CrumbId, style_id: Option<StyleId>) -> Self {
        self.crumbs.push(CrumbItem(crumb_id, IDENTITY, style_id));
        self
    }

    pub fn with_group_id(mut self, group_id: GroupId) -> Self {
        self.groups.push(GroupItem(group_id, IDENTITY));
        self
    }

    pub fn with_crumb_items<I>(mut self, crumbs: I) -> Self
    where
        I: IntoIterator<Item = CrumbItem>,
    {
        self.crumbs.extend(crumbs.into_iter());
        self
    }

    pub fn with_group_items<I>(mut self, groups: I) -> Self
    where
        I: IntoIterator<Item = GroupItem>,
    {
        self.groups.extend(groups.into_iter());
        self
    }

    pub fn with_crumb_ids<I>(mut self, crumbs: I) -> Self
    where
        I: IntoIterator<Item = (CrumbId, Option<StyleId>)>,
    {
        self.crumbs.extend(crumbs.into_iter().map(|(p, s)| CrumbItem(p, IDENTITY, s)));
        self
    }

    pub fn with_group_ids<I>(mut self, groups: I) -> Self
    where
        I: IntoIterator<Item = GroupId>,
    {
        self.groups.extend(groups.into_iter().map(|g| GroupItem(g, IDENTITY)));
        self
    }

    pub fn get_crumb_items(&self) -> &[CrumbItem] {
        self.crumbs.as_slice()
    }

    pub fn get_group_items(&self) -> &[GroupItem] {
        self.groups.as_slice()
    }
}
