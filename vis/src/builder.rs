use kurbo::Vec2;
use crate::{Crumb, CrumbItem, GroupId, StyleId, Scene, TextLabel};

struct NodeLabelEntry {
    index:  usize,
    offset: Vec2,
    name:   String,
    upper:  Option<String>,
    lower:  Option<String>,
}

pub struct NodeLabelBuilder {
    labels: Vec<NodeLabelEntry>,
}

impl NodeLabelBuilder {
    pub fn new<I, J, S>(indices: I, names: J) -> Self
    where
        I: IntoIterator<Item = usize>,
        J: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let labels = indices
            .into_iter()
            .zip(names.into_iter())
            .map(|(index, name)| NodeLabelEntry {
                index,
                offset: Vec2::ZERO,
                name: name.as_ref().to_string(),
                upper: None,
                lower: None,
            })
            .collect();

        NodeLabelBuilder { labels }
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

    pub fn build(&self, nodes_group_id: GroupId, scene: &mut Scene) -> GroupId {
        let labels: Vec<_> = if let Some(group) = scene.get_group(nodes_group_id) {
            NodeLabelIter { scene, crumbs: group.get_crumb_items(), iter: self.labels.iter() }
                .collect()
        } else {
            // FIXME Err
            Vec::new()
        };

        scene.add_grouped_crumbs(labels)
    }
}

struct NodeLabelIter<'a> {
    scene:  &'a Scene,
    crumbs: &'a [CrumbItem],
    iter:   std::slice::Iter<'a, NodeLabelEntry>,
}

impl Iterator for NodeLabelIter<'_> {
    type Item = (Crumb, Option<StyleId>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(NodeLabelEntry { index, offset, name, upper, lower }) = self.iter.next() {
                if let Some(CrumbItem(crumb_id, ..)) = self.crumbs.get(*index) {
                    match self.scene.get_crumb(*crumb_id) {
                        Some(Crumb::Circle(circle)) => {
                            let origin = circle.center + *offset;
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
                        }
                        _ => {}
                    }
                }
            } else {
                return None
            }
        }
    }
}
