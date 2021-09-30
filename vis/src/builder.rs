use kurbo::{Vec2, Point, Circle};
use crate::{Crumb, CrumbId, CrumbItem, Group, GroupId, StyleId, Scene, TextLabel, VisError};

enum NodeRef {
    CrumbInAGroupIndex(usize),
    CrumbId(CrumbId),
    Geometry(Point),
}

struct PinEntry {
    node_ref: NodeRef,
    offset:   Vec2,
}

pub struct PinBuilder {
    name:     Option<String>,
    pins:     Vec<PinEntry>,
    group_id: Option<GroupId>,
}

impl PinBuilder {
    pub fn new() -> Self {
        PinBuilder { name: None, pins: Vec::new(), group_id: None }
    }

    pub fn with_name<S: AsRef<str>>(mut self, name: S) -> Self {
        self.name = Some(name.as_ref().to_string());
        self
    }

    pub fn with_group(mut self, group_id: GroupId) -> Self {
        self.group_id = Some(group_id);
        self
    }

    pub fn with_indices<I>(mut self, indices: I) -> Result<Self, VisError>
    where
        I: IntoIterator<Item = usize>,
    {
        if self.pins.is_empty() {
            self.pins.extend(indices.into_iter().map(|crumb_ndx| PinEntry {
                node_ref: NodeRef::CrumbInAGroupIndex(crumb_ndx),
                offset:   Vec2::ZERO,
            }));
        } else {
            for (pin_ndx, crumb_ndx) in indices.into_iter().enumerate() {
                if let Some(entry) = self.pins.get_mut(pin_ndx) {
                    entry.node_ref = NodeRef::CrumbInAGroupIndex(crumb_ndx);
                } else {
                    return Err(VisError::builder_overflow("pins", self.pins.len()))
                }
            }
        }
        Ok(self)
    }

    pub fn with_crumb_ids<I>(mut self, ids: I) -> Result<Self, VisError>
    where
        I: IntoIterator<Item = CrumbId>,
    {
        if self.pins.is_empty() {
            self.pins.extend(ids.into_iter().map(|crumb_id| PinEntry {
                node_ref: NodeRef::CrumbId(crumb_id),
                offset:   Vec2::ZERO,
            }));
        } else {
            for (pin_ndx, crumb_id) in ids.into_iter().enumerate() {
                if let Some(entry) = self.pins.get_mut(pin_ndx) {
                    entry.node_ref = NodeRef::CrumbId(crumb_id);
                } else {
                    return Err(VisError::builder_overflow("pins", self.pins.len()))
                }
            }
        }
        Ok(self)
    }

    pub fn with_offsets<I, V>(mut self, offsets: I) -> Result<Self, VisError>
    where
        I: IntoIterator<Item = V>,
        V: Into<Vec2>,
    {
        for (ndx, new_offset) in offsets.into_iter().enumerate() {
            if let Some(entry) = self.pins.get_mut(ndx) {
                entry.offset = new_offset.into();
            } else {
                return Err(VisError::builder_overflow("pins", self.pins.len()))
            }
        }
        Ok(self)
    }

    fn resolve_indices(&mut self, scene: &Scene) -> Result<(), VisError> {
        if let Some(group_id) = self.group_id {
            if let Some(group) = scene.get_group(group_id) {
                let crumbs = group.get_crumb_items();

                for entry in &mut self.pins {
                    let crumb_id = match entry.node_ref {
                        NodeRef::CrumbInAGroupIndex(index) => {
                            if let Some(CrumbItem(id, ..)) = crumbs.get(index) {
                                *id
                            } else {
                                return Err(VisError::crumbs_of_a_group_overflow(group_id, index))
                            }
                        }
                        NodeRef::CrumbId(id) => id,
                        NodeRef::Geometry(_) => continue,
                    };

                    match scene.get_crumb(crumb_id) {
                        Some(Crumb::Circle(circle)) => {
                            entry.node_ref = NodeRef::Geometry(circle.center);
                        }
                        Some(crumb) => {
                            return Err(VisError::crumb_mismatch("Circle", crumb.clone(), crumb_id))
                        }
                        None => return Err(VisError::crumb_missing_for_id(crumb_id)),
                    }
                }
            } else {
                return Err(VisError::group_missing_for_id(group_id))
            }
        }
        Ok(())
    }

    pub fn build(&mut self, scene: &mut Scene) -> Result<GroupId, VisError> {
        self.resolve_indices(scene)?;

        let mut group = Group::default();

        if let Some(ref name) = self.name {
            group.set_name(name);
        }

        for pin in (PinIter { entries: self.pins.iter() }) {
            let (crumb, style_id) = pin?;
            let crumb_id = scene.add_crumb(crumb);

            group.add_crumb(crumb_id, style_id);
        }

        Ok(scene.add_group(group))
    }
}

impl Default for PinBuilder {
    fn default() -> Self {
        Self::new()
    }
}

struct PinIter<'a> {
    entries: std::slice::Iter<'a, PinEntry>,
}

impl Iterator for PinIter<'_> {
    type Item = Result<(Crumb, Option<StyleId>), VisError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        #[allow(clippy::collapsible_match)]
        if let Some(PinEntry { node_ref, offset }) = self.entries.next() {
            if let NodeRef::Geometry(center) = node_ref {
                let origin = *center + *offset;
                let pin = Circle::new((origin.x, origin.y), 35.);

                Some(Ok((Crumb::Pin(pin), None)))
            } else {
                Some(Err(VisError::builder_unresolved("Pin")))
            }
        } else {
            None
        }
    }
}

struct NodeLabelEntry {
    node_name: String,
    node_ref:  NodeRef,
    offset:    Vec2,
    upper:     Option<String>,
    lower:     Option<String>,
}

pub struct NodeLabelBuilder {
    name:     Option<String>,
    labels:   Vec<NodeLabelEntry>,
    group_id: Option<GroupId>,
}

impl NodeLabelBuilder {
    pub fn new<I, S>(node_names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let labels = node_names
            .into_iter()
            .enumerate()
            .map(|(ndx, node_name)| NodeLabelEntry {
                node_name: node_name.as_ref().to_string(),
                node_ref:  NodeRef::CrumbInAGroupIndex(ndx),
                offset:    Vec2::ZERO,
                upper:     None,
                lower:     None,
            })
            .collect();

        NodeLabelBuilder { name: None, labels, group_id: None }
    }

    pub fn with_name<S: AsRef<str>>(mut self, name: S) -> Self {
        self.name = Some(name.as_ref().to_string());
        self
    }

    pub fn with_group(mut self, group_id: GroupId) -> Self {
        self.group_id = Some(group_id);
        self
    }

    pub fn with_indices<I>(mut self, indices: I) -> Result<Self, VisError>
    where
        I: IntoIterator<Item = usize>,
    {
        for (label_ndx, crumb_ndx) in indices.into_iter().enumerate() {
            if let Some(entry) = self.labels.get_mut(label_ndx) {
                entry.node_ref = NodeRef::CrumbInAGroupIndex(crumb_ndx);
            } else {
                return Err(VisError::builder_overflow("labels", self.labels.len()))
            }
        }
        Ok(self)
    }

    pub fn with_crumb_ids<I>(mut self, ids: I) -> Result<Self, VisError>
    where
        I: IntoIterator<Item = CrumbId>,
    {
        for (label_ndx, crumb_id) in ids.into_iter().enumerate() {
            if let Some(entry) = self.labels.get_mut(label_ndx) {
                entry.node_ref = NodeRef::CrumbId(crumb_id);
            } else {
                return Err(VisError::builder_overflow("labels", self.labels.len()))
            }
        }
        Ok(self)
    }

    pub fn with_offsets<I, V>(mut self, offsets: I) -> Result<Self, VisError>
    where
        I: IntoIterator<Item = V>,
        V: Into<Vec2>,
    {
        for (ndx, new_offset) in offsets.into_iter().enumerate() {
            if let Some(entry) = self.labels.get_mut(ndx) {
                entry.offset = new_offset.into();
            } else {
                return Err(VisError::builder_overflow("labels", self.labels.len()))
            }
        }
        Ok(self)
    }

    pub fn with_spans<I, J, S, T>(mut self, upper: I, lower: J) -> Result<Self, VisError>
    where
        I: IntoIterator<Item = S>,
        J: IntoIterator<Item = T>,
        S: AsRef<str>,
        T: AsRef<str>,
    {
        for (ndx, span) in upper.into_iter().enumerate() {
            if let Some(entry) = self.labels.get_mut(ndx) {
                let span = span.as_ref();

                if span.is_empty() {
                    entry.upper = None;
                } else {
                    entry.upper = Some(span.to_string());
                }
            } else {
                return Err(VisError::builder_overflow("labels", self.labels.len()))
            }
        }
        for (ndx, span) in lower.into_iter().enumerate() {
            if let Some(entry) = self.labels.get_mut(ndx) {
                let span = span.as_ref();

                if span.is_empty() {
                    entry.lower = None;
                } else {
                    entry.lower = Some(span.to_string());
                }
            } else {
                return Err(VisError::builder_overflow("labels", self.labels.len()))
            }
        }
        Ok(self)
    }

    fn resolve_indices(&mut self, scene: &Scene) -> Result<(), VisError> {
        if let Some(group_id) = self.group_id {
            if let Some(group) = scene.get_group(group_id) {
                let crumbs = group.get_crumb_items();

                for entry in &mut self.labels {
                    let crumb_id = match entry.node_ref {
                        NodeRef::CrumbInAGroupIndex(index) => {
                            if let Some(CrumbItem(id, ..)) = crumbs.get(index) {
                                *id
                            } else {
                                return Err(VisError::crumbs_of_a_group_overflow(group_id, index))
                            }
                        }
                        NodeRef::CrumbId(id) => id,
                        NodeRef::Geometry(_) => continue,
                    };

                    match scene.get_crumb(crumb_id) {
                        Some(Crumb::Circle(circle)) => {
                            entry.node_ref = NodeRef::Geometry(circle.center);
                        }
                        Some(crumb) => {
                            return Err(VisError::crumb_mismatch("Circle", crumb.clone(), crumb_id))
                        }
                        None => return Err(VisError::crumb_missing_for_id(crumb_id)),
                    }
                }
            } else {
                return Err(VisError::group_missing_for_id(group_id))
            }
        }
        Ok(())
    }

    pub fn build(&mut self, scene: &mut Scene) -> Result<GroupId, VisError> {
        self.resolve_indices(scene)?;

        let mut group = Group::default();

        if let Some(ref name) = self.name {
            group.set_name(name);
        }

        for label in (NodeLabelIter { entries: self.labels.iter() }) {
            let (crumb, style_id) = label?;
            let crumb_id = scene.add_crumb(crumb);

            group.add_crumb(crumb_id, style_id);
        }

        Ok(scene.add_group(group))
    }
}

struct NodeLabelIter<'a> {
    entries: std::slice::Iter<'a, NodeLabelEntry>,
}

impl Iterator for NodeLabelIter<'_> {
    type Item = Result<(Crumb, Option<StyleId>), VisError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        #[allow(clippy::collapsible_match)]
        if let Some(NodeLabelEntry { node_name, node_ref, offset, upper, lower }) =
            self.entries.next()
        {
            if let NodeRef::Geometry(center) = node_ref {
                let origin = *center + *offset;
                let mut label = TextLabel::new()
                    .with_text(node_name)
                    .with_end_anchor()
                    .with_origin(origin)
                    .with_font_size(28.0);

                if let Some(upper) = upper {
                    label.append_span(
                        TextLabel::new()
                            .with_text(upper)
                            .with_origin(origin)
                            .with_dy([-16.0])
                            .with_font_size(22.0),
                    );
                }

                if let Some(lower) = lower {
                    label.append_span(
                        TextLabel::new()
                            .with_text(lower)
                            .with_origin(origin)
                            .with_dy([12.0])
                            .with_font_size(22.0),
                    );
                }

                Some(Ok((Crumb::Label(label), None)))
            } else {
                Some(Err(VisError::builder_unresolved("Label")))
            }
        } else {
            None
        }
    }
}
