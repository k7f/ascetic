use kurbo::{Vec2, Point, Circle};
use crate::{Crumb, CrumbId, CrumbItem, GroupId, StyleId, Scene, TextLabel};

enum NodeRef {
    GroupIndex(usize),
    CrumbId(CrumbId),
    Geometry(Point),
}

struct PinEntry {
    node:   NodeRef,
    offset: Vec2,
}

pub struct PinBuilder {
    pins:     Vec<PinEntry>,
    group_id: Option<GroupId>,
}

impl PinBuilder {
    pub fn new() -> Self {
        PinBuilder { pins: Vec::new(), group_id: None }
    }

    pub fn with_group(mut self, group_id: GroupId) -> Self {
        self.group_id = Some(group_id);
        self
    }

    pub fn with_indices<I>(mut self, indices: I) -> Self
    where
        I: IntoIterator<Item = usize>,
    {
        if self.pins.is_empty() {
            self.pins.extend(indices.into_iter().map(|crumb_ndx| PinEntry {
                node:   NodeRef::GroupIndex(crumb_ndx),
                offset: Vec2::ZERO,
            }));
        } else {
            for (pin_ndx, crumb_ndx) in indices.into_iter().enumerate() {
                if let Some(entry) = self.pins.get_mut(pin_ndx) {
                    entry.node = NodeRef::GroupIndex(crumb_ndx);
                } else {
                    // FIXME Err
                    break
                }
            }
        }
        self
    }

    pub fn with_crumb_ids<I>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = CrumbId>,
    {
        if self.pins.is_empty() {
            self.pins.extend(ids.into_iter().map(|crumb_id| PinEntry {
                node:   NodeRef::CrumbId(crumb_id),
                offset: Vec2::ZERO,
            }));
        } else {
            for (pin_ndx, crumb_id) in ids.into_iter().enumerate() {
                if let Some(entry) = self.pins.get_mut(pin_ndx) {
                    entry.node = NodeRef::CrumbId(crumb_id);
                } else {
                    // FIXME Err
                    break
                }
            }
        }
        self
    }

    pub fn with_offsets<I, V>(mut self, offsets: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Into<Vec2>,
    {
        for (ndx, new_offset) in offsets.into_iter().enumerate() {
            if let Some(entry) = self.pins.get_mut(ndx) {
                entry.offset = new_offset.into();
            } else {
                // FIXME Err
                break
            }
        }
        self
    }

    fn resolve_indices(&mut self, scene: &Scene) {
        if let Some(group_id) = self.group_id {
            if let Some(group) = scene.get_group(group_id) {
                let crumbs = group.get_crumb_items();

                for entry in &mut self.pins {
                    let crumb_id = match entry.node {
                        NodeRef::GroupIndex(index) => {
                            if let Some(CrumbItem(id, ..)) = crumbs.get(index) {
                                *id
                            } else {
                                // FIXME Err
                                break
                            }
                        }
                        NodeRef::CrumbId(id) => id,
                        NodeRef::Geometry(_) => continue,
                    };

                    match scene.get_crumb(crumb_id) {
                        Some(Crumb::Circle(circle)) => {
                            entry.node = NodeRef::Geometry(circle.center)
                        }
                        _ => {
                            // FIXME
                        }
                    }
                }
            } else {
                // FIXME Err
            }
        }
    }

    pub fn build(&mut self, scene: &mut Scene) -> GroupId {
        self.resolve_indices(scene);

        let pins = PinIter { entries: self.pins.iter() };

        scene.add_grouped_crumbs(pins)
    }
}

struct PinIter<'a> {
    entries: std::slice::Iter<'a, PinEntry>,
}

impl Iterator for PinIter<'_> {
    type Item = (Crumb, Option<StyleId>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(PinEntry { node, offset }) = self.entries.next() {
                if let NodeRef::Geometry(center) = node {
                    let origin = *center + *offset;
                    let pin = Circle::new((origin.x, origin.y), 35.);

                    return Some((Crumb::Pin(pin), None))
                } else {
                    // FIXME error unresolved index
                }
            } else {
                return None
            }
        }
    }
}

struct NodeLabelEntry {
    name:   String,
    node:   NodeRef,
    offset: Vec2,
    upper:  Option<String>,
    lower:  Option<String>,
}

pub struct NodeLabelBuilder {
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
            .map(|(ndx, name)| NodeLabelEntry {
                name:   name.as_ref().to_string(),
                node:   NodeRef::GroupIndex(ndx),
                offset: Vec2::ZERO,
                upper:  None,
                lower:  None,
            })
            .collect();

        NodeLabelBuilder { labels, group_id: None }
    }

    pub fn with_group(mut self, group_id: GroupId) -> Self {
        self.group_id = Some(group_id);
        self
    }

    pub fn with_indices<I>(mut self, indices: I) -> Self
    where
        I: IntoIterator<Item = usize>,
    {
        for (label_ndx, crumb_ndx) in indices.into_iter().enumerate() {
            if let Some(entry) = self.labels.get_mut(label_ndx) {
                entry.node = NodeRef::GroupIndex(crumb_ndx);
            } else {
                // FIXME Err
                break
            }
        }
        self
    }

    pub fn with_crumb_ids<I>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = CrumbId>,
    {
        for (label_ndx, crumb_id) in ids.into_iter().enumerate() {
            if let Some(entry) = self.labels.get_mut(label_ndx) {
                entry.node = NodeRef::CrumbId(crumb_id);
            } else {
                // FIXME Err
                break
            }
        }
        self
    }

    pub fn with_offsets<I, V>(mut self, offsets: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Into<Vec2>,
    {
        for (ndx, new_offset) in offsets.into_iter().enumerate() {
            if let Some(entry) = self.labels.get_mut(ndx) {
                entry.offset = new_offset.into();
            } else {
                // FIXME Err
                break
            }
        }
        self
    }

    pub fn with_spans<I, J, S, T>(mut self, upper: I, lower: J) -> Self
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
                // FIXME Err
                break
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
                // FIXME Err
                break
            }
        }
        self
    }

    fn resolve_indices(&mut self, scene: &Scene) {
        if let Some(group_id) = self.group_id {
            if let Some(group) = scene.get_group(group_id) {
                let crumbs = group.get_crumb_items();

                for entry in &mut self.labels {
                    let crumb_id = match entry.node {
                        NodeRef::GroupIndex(index) => {
                            if let Some(CrumbItem(id, ..)) = crumbs.get(index) {
                                *id
                            } else {
                                // FIXME Err
                                break
                            }
                        }
                        NodeRef::CrumbId(id) => id,
                        NodeRef::Geometry(_) => continue,
                    };

                    match scene.get_crumb(crumb_id) {
                        Some(Crumb::Circle(circle)) => {
                            entry.node = NodeRef::Geometry(circle.center)
                        }
                        _ => {
                            // FIXME
                        }
                    }
                }
            } else {
                // FIXME Err
            }
        }
    }

    pub fn build(&mut self, scene: &mut Scene) -> GroupId {
        self.resolve_indices(scene);

        let labels = NodeLabelIter { entries: self.labels.iter() };

        scene.add_grouped_crumbs(labels)
    }
}

struct NodeLabelIter<'a> {
    entries: std::slice::Iter<'a, NodeLabelEntry>,
}

impl Iterator for NodeLabelIter<'_> {
    type Item = (Crumb, Option<StyleId>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(NodeLabelEntry { name, node, offset, upper, lower }) = self.entries.next() {
                if let NodeRef::Geometry(center) = node {
                    let origin = *center + *offset;
                    let mut label = TextLabel::new()
                        .with_text(name)
                        .with_end_anchor()
                        .with_origin(origin)
                        .with_font_size(28.0);

                    if let Some(upper) = upper {
                        label.append_span(
                            TextLabel::new()
                                .with_text(upper)
                                .with_origin(origin)
                                .with_dy([-10.0])
                                .with_font_size(22.0),
                        );
                    }

                    if let Some(lower) = lower {
                        label.append_span(
                            TextLabel::new()
                                .with_text(lower)
                                .with_origin(origin)
                                .with_dy([10.0])
                                .with_font_size(22.0),
                        );
                    }

                    return Some((Crumb::Label(label), None))
                } else {
                    // FIXME error unresolved index
                }
            } else {
                return None
            }
        }
    }
}
