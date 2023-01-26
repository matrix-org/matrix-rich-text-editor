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
use std::collections::HashSet;

use crate::dom::nodes::dom_node::DomNodeKind;
use crate::dom::nodes::dom_node::DomNodeKind::{Link, List};
use crate::dom::nodes::dom_node::{DomNodeKind::LineBreak, DomNodeKind::Text};
use crate::dom::nodes::ContainerNodeKind;
use crate::dom::nodes::DomNode;
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
        for loc in range.locations.iter() {
            if loc.kind == DomNodeKind::Link {
                let node = self.state.dom.lookup_node(&loc.node_handle);
                let link = node.as_container().unwrap().get_link().unwrap();
                return LinkAction::Edit(link);
            }
        }
        if s == e || self.is_blank_selection(range) {
            LinkAction::CreateWithText
        } else {
            LinkAction::Create
        }
    }

    fn is_blank_selection(&self, range: Range) -> bool {
        for leaf in range.leaves() {
            if leaf.kind == LineBreak {
                continue;
            } else if leaf.kind == Text {
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
            } else {
                return false;
            }
        }
        true
    }

    pub fn set_link_with_text(
        &mut self,
        link: S,
        text: S,
    ) -> ComposerUpdate<S> {
        let (s, _) = self.safe_selection();
        self.push_state_to_history();
        self.do_replace_text(text.clone());
        let e = s + text.len();
        let range = self.state.dom.find_range(s, e);
        self.set_link_in_range(link, range)
    }

    pub fn set_link(&mut self, link: S) -> ComposerUpdate<S> {
        self.push_state_to_history();
        let (s, e) = self.safe_selection();

        let range = self.state.dom.find_range(s, e);

        self.set_link_in_range(link, range)
    }

    fn set_link_in_range(
        &mut self,
        mut link: S,
        range: Range,
    ) -> ComposerUpdate<S> {
        self.add_http_scheme(&mut link);

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
            if (location.kind.is_block_kind()
                || location.kind.is_structure_kind())
                && location.kind != List
            {
                let start = location.position + location.start_offset;
                let end = if location.end_offset == location.length
                    && !location.node_handle.is_root()
                {
                    location.position + location.end_offset - 1
                } else {
                    location.position + location.end_offset
                };
                if !split_points
                    .iter()
                    .any(|(h, _, _)| location.node_handle.is_ancestor_of(h))
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
            if location.kind == Link {
                let start = location.position;
                let end = location.position + location.length;
                let idx = split_points.iter().position(|(h, s, e)| {
                    h.is_ancestor_of(&location.node_handle)
                        || (*s <= start && *e >= end)
                });
                if let Some(idx) = idx {
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
                    split_points.push((
                        location.node_handle.clone(),
                        start,
                        end,
                    ));
                }
            }
        }

        if split_points.is_empty() {
            split_points.push((DomHandle::root(), s, e));
        }

        for (handle, s, e) in split_points.into_iter() {
            let range = self.state.dom.find_range(s, e);
            let inserted = self
                .state
                .dom
                .insert_parent(&range, DomNode::new_link(link.clone(), vec![]));
            self.delete_child_links(&inserted);
        }

        self.create_update_replace_all()
    }

    fn add_http_scheme(&mut self, link: &mut S) {
        let string = link.to_string();
        let str = string.as_str();

        match Url::parse(str) {
            Ok(_) => {}
            Err(ParseError::RelativeUrlWithoutBase) => {
                let is_email = EmailAddress::is_valid(str);

                if is_email {
                    link.insert(0, &S::from("mailto:"));
                } else {
                    link.insert(0, &S::from("https://"));
                };
            }
            Err(_) => {}
        };
    }

    fn delete_child_links(&mut self, node_handle: &DomHandle) {
        let node = self.state.dom.lookup_node(node_handle);

        let child_link_handles = node
            .iter_containers()
            .filter(|n| matches!(n.kind(), ContainerNodeKind::Link(_)))
            .map(|n| n.handle())
            .filter(|h| *h != *node_handle)
            .collect::<Vec<_>>();

        for handle in child_link_handles.into_iter().rev() {
            self.state.dom.remove_and_keep_children(&handle);
        }
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
