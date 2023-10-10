// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::cmp::{max, min};

use crate::dom::nodes::dom_node::DomNodeKind;
use crate::dom::nodes::dom_node::DomNodeKind::{Link, List};
use crate::dom::nodes::ContainerNodeKind;
use crate::dom::nodes::DomNode;
use crate::dom::to_plain_text::ToPlainText;
use crate::dom::unicode_string::UnicodeStrExt;
use crate::dom::Range;
use crate::{
    ComposerModel, ComposerUpdate, DomHandle, LinkAction, UnicodeString,
};
use email_address::*;
use url::{ParseError, Url};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn get_link_action(&self) -> LinkAction<S> {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        let mut iter = range
            .locations
            .iter()
            .filter(|loc| loc.kind == DomNodeKind::Link);

        if let Some(first_loc) = iter.next() {
            let first_link =
                self.state.dom.lookup_container(&first_loc.node_handle);
            // If any of the link in the selection is immutable, link actions are disabled.
            if first_link.is_immutable()
                || iter.any(|loc| {
                    self.state
                        .dom
                        .lookup_container(&loc.node_handle)
                        .is_immutable()
                })
            {
                LinkAction::Disabled
            } else {
                // Otherwise we edit the first link of the selection.
                LinkAction::Edit(
                    first_link.get_link_url().unwrap(),
                    first_link.to_plain_text(),
                )
            }
        } else if s == e || self.is_blank_selection(range) {
            LinkAction::CreateWithText
        } else {
            LinkAction::Create
        }
    }

    fn is_blank_selection(&self, range: Range) -> bool {
        for leaf in range.leaves() {
            match leaf.kind {
                DomNodeKind::Text => {
                    let text_node = self
                        .state
                        .dom
                        .lookup_node(&leaf.node_handle)
                        .as_text()
                        .unwrap();
                    let selection_range = leaf.start_offset..leaf.end_offset;
                    if !text_node.is_blank_in_range(selection_range) {
                        return false;
                    }
                }
                DomNodeKind::LineBreak => continue,
                DomNodeKind::Mention => return false,
                DomNodeKind::Formatting(_)
                | DomNodeKind::Link
                | DomNodeKind::ListItem
                | DomNodeKind::List
                | DomNodeKind::CodeBlock
                | DomNodeKind::Quote
                | DomNodeKind::Generic
                | DomNodeKind::Paragraph => {
                    unreachable!("Inside leaf iterator and found a non-leaf")
                }
            }
        }
        true
    }

    pub fn set_link_with_text(
        &mut self,
        url: S,
        text: S,
        attributes: Vec<(S, S)>,
    ) -> ComposerUpdate<S> {
        let (s, _) = self.safe_selection();
        self.push_state_to_history();
        self.do_replace_text(text.clone());
        let e = s + text.len();
        let range = self.state.dom.find_range(s, e);
        self.set_link_in_range(url, range, attributes)
    }

    pub fn set_link(
        &mut self,
        url: S,
        attributes: Vec<(S, S)>,
    ) -> ComposerUpdate<S> {
        self.push_state_to_history();
        let (s, e) = self.safe_selection();

        let range = self.state.dom.find_range(s, e);

        self.set_link_in_range(url, range, attributes)
    }

    pub fn edit_link_with_text(
        &mut self,
        url: S,
        text: S,
        attributes: Vec<(S, S)>,
    ) -> ComposerUpdate<S> {
        self.push_state_to_history();
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        let Some(link_loc) = range
            .locations
            .iter()
            .find(|loc| loc.kind == DomNodeKind::Link) else {
                panic!("Attempting to edit a link on a range that doesn't contain one")
        };
        let start = link_loc.position;
        let end = start + link_loc.length;
        let new_end = start + text.len();
        self.do_replace_text_in(text, start, end);
        let range = self.state.dom.find_range(start, new_end);
        self.set_link_in_range(url, range, attributes)
    }

    fn set_link_in_range(
        &mut self,
        mut url: S,
        range: Range,
        attributes: Vec<(S, S)>,
    ) -> ComposerUpdate<S> {
        self.add_http_scheme(&mut url);

        let (mut s, mut e) = (range.start(), range.end());
        // Find container link that completely covers the range
        if let Some(link) = self.find_closest_ancestor_link(&range) {
            // If found, update the range to the container link bounds
            let range = self.state.dom.find_range_by_node(&link);
            (s, e) = (range.start(), range.end());
        }

        if s == e {
            return ComposerUpdate::keep();
        }

        let mut split_points: Vec<(DomHandle, usize, usize)> = Vec::new();

        for location in range.locations.iter() {
            // Look for block nodes
            if (location.kind.is_block_kind()
                || location.kind.is_structure_kind())
                && location.kind != List
            {
                let start = location.position + location.start_offset;
                let end = if location.end_offset == location.length
                    && !location.node_handle.is_root()
                {
                    // The end of the block node is covered (end_offset == length), don't include it
                    location.position + location.end_offset - 1
                } else {
                    location.position + location.end_offset
                };
                // If there was a child block node added as a split point, don't add this one
                if !split_points
                    .iter()
                    .any(|(h, _, _)| location.node_handle.is_ancestor_of(h))
                    // Only include split points which actually have some text in the range.
                    && end > start
                {
                    split_points.push((
                        location.node_handle.clone(),
                        start,
                        end,
                    ));
                }
            }
        }

        for location in range.locations.iter() {
            // Now look for previous links inside the selection
            if location.kind == Link {
                let start = location.position;
                let end = location.position + location.length;
                let idx = split_points.iter().position(|(h, s, e)| {
                    h.is_ancestor_of(&location.node_handle)
                        || (*s <= start && *e >= end)
                });
                if let Some(idx) = idx {
                    // If a parent or intersecting node was added before, remove it and extend this
                    // one to match it (i.e., another link was already added).
                    let (_, s, e) = split_points.remove(idx);
                    split_points.insert(
                        idx,
                        (
                            location.node_handle.clone(),
                            min(s, start),
                            max(e, end),
                        ),
                    );
                } else {
                    // Otherwise, just add another split point.
                    split_points.push((
                        location.node_handle.clone(),
                        start,
                        end,
                    ));
                }
            }
        }

        for (_, s, e) in split_points.into_iter() {
            let range = self.state.dom.find_range(s, e);

            // Create a new link node containing the passed range
            let inserted = self.state.dom.insert_parent(
                &range,
                DomNode::new_link(url.clone(), vec![], attributes.clone()),
            );

            // Remove any child links or mentions inside it
            self.delete_child_links(&inserted);
            self.convert_child_mentions_to_text(&inserted);
        }

        self.create_update_replace_all()
    }

    fn add_http_scheme(&mut self, url: &mut S) {
        let string = url.to_string();
        let str = string.as_str();

        match Url::parse(str) {
            Ok(_) => {}
            Err(ParseError::RelativeUrlWithoutBase) => {
                let is_email = EmailAddress::is_valid(str);

                if is_email {
                    url.insert(0, &S::from("mailto:"));
                } else {
                    url.insert(0, &S::from("https://"));
                };
            }
            Err(_) => {}
        };
    }

    fn delete_child_links(&mut self, node_handle: &DomHandle) {
        let node = self.state.dom.lookup_node(node_handle);

        node.iter_containers()
            .filter_map(|c| {
                if c.is_link() && c.handle() != *node_handle {
                    Some(c.handle())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .for_each(|h| self.state.dom.remove_and_keep_children(&h));
    }

    fn convert_child_mentions_to_text(&mut self, node_handle: &DomHandle) {
        self.state
            .dom
            .lookup_node(node_handle)
            .iter_subtree()
            .filter_map(|node| match node {
                DomNode::Mention(node) => {
                    Some((node.handle(), node.display_text()))
                }
                _ => None,
            })
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .for_each(|(handle, display_text)| {
                self.state.dom.replace(
                    &handle,
                    vec![DomNode::new_text(display_text.clone())],
                );
                let selection_length_change: isize =
                    (display_text.len() - 1).try_into().unwrap_or(0);
                self.state.extend_selection(selection_length_change)
            });
    }

    fn find_closest_ancestor_link(
        &mut self,
        range: &Range,
    ) -> Option<DomHandle> {
        let mut parent_handle = range.shared_parent_outside();
        while !parent_handle.is_root() {
            let node = self.state.dom.lookup_node(&parent_handle);
            let container = match node {
                DomNode::Container(container) => container,
                _ => continue,
            };
            if matches!(container.kind(), ContainerNodeKind::Link(_)) {
                return Some(node.handle());
            }
            parent_handle = parent_handle.parent_handle();
        }

        None
    }

    pub fn remove_links(&mut self) -> ComposerUpdate<S> {
        let mut has_found_link = false;
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        let iter = range.locations.into_iter().rev();
        for loc in iter {
            if loc.kind == DomNodeKind::Link {
                if !has_found_link {
                    has_found_link = true;
                    self.push_state_to_history();
                }
                self.state
                    .dom
                    .replace_node_with_its_children(&loc.node_handle);
            }
        }
        if !has_found_link {
            return ComposerUpdate::keep();
        }
        self.create_update_replace_all()
    }
}
