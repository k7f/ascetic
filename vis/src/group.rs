use piet_common::kurbo::TranslateScale;
use crate::{PrimId, GroupId, StyleId};

const IDENTITY: TranslateScale = TranslateScale::scale(1.0);

/// `Groups` are plain containers with neither drawing nor
/// transformation capabilities.
#[derive(Clone, Debug)]
pub struct Group {
    prims:  Vec<(PrimId, Option<StyleId>, TranslateScale)>,
    groups: Vec<(GroupId, TranslateScale)>,
}

impl Group {
    pub fn from_prims_ts<I>(prims: I) -> Self
    where
        I: IntoIterator<Item = (PrimId, Option<StyleId>, TranslateScale)>,
    {
        let prims = prims.into_iter().collect();
        let groups = Vec::new();

        Group { prims, groups }
    }

    pub fn from_groups_ts<I>(groups: I) -> Self
    where
        I: IntoIterator<Item = (GroupId, TranslateScale)>,
    {
        let prims = Vec::new();
        let groups = groups.into_iter().collect();

        Group { prims, groups }
    }

    pub fn from_prims<I>(prims: I) -> Self
    where
        I: IntoIterator<Item = (PrimId, Option<StyleId>)>,
    {
        let prims = prims.into_iter().map(|(p, s)| (p, s, IDENTITY)).collect();
        let groups = Vec::new();

        Group { prims, groups }
    }

    pub fn from_groups<I>(groups: I) -> Self
    where
        I: IntoIterator<Item = GroupId>,
    {
        let prims = Vec::new();
        let groups = groups.into_iter().map(|g| (g, IDENTITY)).collect();

        Group { prims, groups }
    }

    pub fn with_prim_ts(
        mut self,
        prim_id: PrimId,
        style_id: Option<StyleId>,
        ts: TranslateScale,
    ) -> Self {
        self.prims.push((prim_id, style_id, ts));
        self
    }

    pub fn with_group_ts(mut self, group_id: GroupId, ts: TranslateScale) -> Self {
        self.groups.push((group_id, ts));
        self
    }

    pub fn with_prim(mut self, prim_id: PrimId, style_id: Option<StyleId>) -> Self {
        self.prims.push((prim_id, style_id, IDENTITY));
        self
    }

    pub fn with_group(mut self, group_id: GroupId) -> Self {
        self.groups.push((group_id, IDENTITY));
        self
    }

    pub fn with_prims_ts<I>(mut self, prims: I) -> Self
    where
        I: IntoIterator<Item = (PrimId, Option<StyleId>, TranslateScale)>,
    {
        self.prims.extend(prims.into_iter());
        self
    }

    pub fn with_groups_ts<I>(mut self, groups: I) -> Self
    where
        I: IntoIterator<Item = (GroupId, TranslateScale)>,
    {
        self.groups.extend(groups.into_iter());
        self
    }

    pub fn with_prims<I>(mut self, prims: I) -> Self
    where
        I: IntoIterator<Item = (PrimId, Option<StyleId>)>,
    {
        self.prims.extend(prims.into_iter().map(|(p, s)| (p, s, IDENTITY)));
        self
    }

    pub fn with_groups<I>(mut self, groups: I) -> Self
    where
        I: IntoIterator<Item = GroupId>,
    {
        self.groups.extend(groups.into_iter().map(|g| (g, IDENTITY)));
        self
    }

    pub fn get_prims(&self) -> &[(PrimId, Option<StyleId>, TranslateScale)] {
        self.prims.as_slice()
    }

    pub fn get_groups(&self) -> &[(GroupId, TranslateScale)] {
        self.groups.as_slice()
    }
}
