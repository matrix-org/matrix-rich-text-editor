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

use crate::dom::nodes::{ContainerNodeKind, DomNode};
use crate::dom::unicode_string::UnicodeStrExt;
use crate::dom::{Dom, DomHandle, DomLocation, Range};
use crate::{
    ComposerAction, ComposerModel, ComposerUpdate, InlineFormatType, Location,
    UnicodeString,
};
use std::collections::HashMap;

#[derive(Eq, PartialEq, Debug)]
enum FormatSelectionType {
    Extend,
    Remove,
}

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn bold(&mut self) -> ComposerUpdate<S> {
        if self.reversed_actions.contains(&ComposerAction::Bold) {
            self.unformat(InlineFormatType::Bold)
        } else {
            self.format(InlineFormatType::Bold)
        }
    }

    pub fn italic(&mut self) -> ComposerUpdate<S> {
        if self.reversed_actions.contains(&ComposerAction::Italic) {
            self.unformat(InlineFormatType::Italic)
        } else {
            self.format(InlineFormatType::Italic)
        }
    }

    pub fn strike_through(&mut self) -> ComposerUpdate<S> {
        if self
            .reversed_actions
            .contains(&ComposerAction::StrikeThrough)
        {
            self.unformat(InlineFormatType::StrikeThrough)
        } else {
            self.format(InlineFormatType::StrikeThrough)
        }
    }

    pub fn underline(&mut self) -> ComposerUpdate<S> {
        if self.reversed_actions.contains(&ComposerAction::Underline) {
            self.unformat(InlineFormatType::Underline)
        } else {
            self.format(InlineFormatType::Underline)
        }
    }

    pub fn inline_code(&mut self) -> ComposerUpdate<S> {
        if self.reversed_actions.contains(&ComposerAction::InlineCode) {
            self.unformat(InlineFormatType::InlineCode)
        } else {
            self.format(InlineFormatType::InlineCode)
        }
    }

    pub fn format(&mut self, format: InlineFormatType) -> ComposerUpdate<S> {
        // Store current Dom
        self.push_state_to_history();
        let (s, e) = self.safe_selection();
        self.format_range(s, e, format);
        self.create_update_replace_all()
    }

    fn format_range(
        &mut self,
        start: usize,
        end: usize,
        format: InlineFormatType,
    ) {
        let range = self.state.dom.find_range(start, end);
        self.format_several_nodes(&range, format);
    }

    pub fn unformat(&mut self, format: InlineFormatType) -> ComposerUpdate<S> {
        // Store current Dom
        self.push_state_to_history();
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        self.unformat_several_nodes(s, e, &range, format);
        self.create_update_replace_all()
    }

    fn check_format_selection_type(
        &self,
        locations: &[DomLocation],
        format: &InlineFormatType,
    ) -> FormatSelectionType {
        let any_format_node = locations.iter().any(|l| {
            let node = self.state.dom.lookup_node(&l.node_handle);
            Self::is_format_node(node, format)
        });

        // If there are no format nodes, we can only create new formatting nodes by Extend
        if !any_format_node {
            FormatSelectionType::Extend
        } else {
            // Find text nodes inside the selection that are not formatted with this format
            let non_formatted_leaf_locations = locations.iter().filter(|l| {
                l.is_leaf
                    && Self::path_contains_format_node(
                        &self.state.dom,
                        &l.node_handle,
                        format,
                    )
                    .is_none()
            });
            // If there are selected non-formatted text nodes, we need to extend the format to them
            let is_extend = non_formatted_leaf_locations.count() > 0;
            if is_extend {
                FormatSelectionType::Extend
            } else {
                // Otherwise, we found only format notes partially or totally covered by the
                // selection, we need to remove formatting in the selection range
                FormatSelectionType::Remove
            }
        }
    }

    fn format_several_nodes(
        &mut self,
        range: &Range,
        format: InlineFormatType,
    ) {
        let selection_type =
            self.check_format_selection_type(&range.locations, &format);
        match selection_type {
            FormatSelectionType::Remove => {} // TODO: actually implement this
            FormatSelectionType::Extend => self
                .extend_format_in_multiple_nodes(
                    range.leaves().collect(),
                    &format,
                ),
        }
    }

    fn unformat_several_nodes(
        &mut self,
        start: usize,
        end: usize,
        range: &Range,
        format: InlineFormatType,
    ) {
        for location in range.locations.iter() {
            let node = self.state.dom.lookup_node(&location.node_handle);
            if let DomNode::Container(n) = node {
                if let ContainerNodeKind::Formatting(f) = n.kind() {
                    if *f == format {
                        if node.has_only_placeholder_text_child() {
                            self.state.end = self.state.start;
                            self.state
                                .dom
                                .replace(&location.node_handle, vec![]);
                        } else {
                            self.state.dom.remove_and_keep_children(
                                &location.node_handle,
                            );
                        }

                        // Re-apply formatting to slices before and after the selection if needed
                        let (before_start, before_end) = self
                            .safe_locations_from(
                                Location::from(start - location.start_offset),
                                Location::from(start),
                            );
                        let (after_start, after_end) = self
                            .safe_locations_from(
                                Location::from(end),
                                Location::from(
                                    end + location.length - location.end_offset,
                                ),
                            );

                        if before_end > before_start {
                            self.format_range(
                                before_start,
                                before_end,
                                format.clone(),
                            );
                        }
                        if after_end > after_start {
                            self.format_range(
                                after_start,
                                after_end,
                                format.clone(),
                            );
                        }
                    }
                }
            }
        }
    }

    fn needs_format(
        dom: &Dom<S>,
        loc: &DomLocation,
        format: &InlineFormatType,
    ) -> bool {
        Self::path_contains_format_node(dom, &loc.node_handle, format).is_none()
    }

    fn extend_format_in_multiple_nodes(
        &mut self,
        locations: Vec<&DomLocation>,
        format: &InlineFormatType,
    ) {
        let mut moved_handles = Vec::<(DomHandle, DomHandle)>::new();
        let mut sorted_locations = locations;
        sorted_locations.sort();
        // Go through the locations in reverse order to prevent Dom modification issues
        for loc in sorted_locations.into_iter().rev() {
            let mut loc = loc.clone();
            let moved_handle = moved_handles
                .iter()
                .find(|(old, _)| old.is_parent_of(&loc.node_handle));
            if let Some((old_handle, new_handle)) = moved_handle {
                // Careful here, the location's position is no longer valid
                let mut new_path = loc.node_handle.clone().into_raw();
                new_path.splice(
                    0..old_handle.raw().len(),
                    new_handle.clone().into_raw(),
                );
                loc = loc.with_new_handle(DomHandle::from_raw(new_path));
            }
            if Self::needs_format(&self.state.dom, &loc, format) {
                if let DomNode::Container(parent) = self
                    .state
                    .dom
                    .lookup_node_mut(&loc.node_handle.parent_handle())
                {
                    let index = loc.node_handle.index_in_parent();
                    let node = parent.remove_child(index);
                    if loc.is_covered() {
                        // Node completely covered by selection, happy path. Just replace the old
                        // text node with a formatting node that contains a copy.
                        let format_node =
                            DomNode::new_formatting(format.clone(), vec![node]);
                        parent.insert_child(index, format_node);
                    } else {
                        // Node only partially covered by selection, we need
                        // to split into 2 or 3 nodes and add them to the
                        // parent.
                        let (before, mut middle, after) =
                            Self::split_text_node_by_offsets(&loc, node);

                        if let Some(after) = after {
                            parent.insert_child(index, after);
                        }
                        self.state.end +=
                            Self::insert_zwspace_if_needed(&mut middle);
                        let middle = DomNode::new_formatting(
                            format.clone(),
                            vec![middle],
                        );
                        parent.insert_child(index, middle);

                        if let Some(before) = before {
                            parent.insert_child(index, before);
                        }
                    }
                    // Clean up by removing any empty text nodes and merging formatting nodes
                    moved_handles.extend(
                        self.merge_formatting_node_with_siblings(
                            &loc.node_handle,
                        ),
                    );
                } else {
                    panic!("Parent is not a container!");
                }
            }
        }
    }

    fn path_contains_format_node(
        dom: &Dom<S>,
        handle: &DomHandle,
        format: &InlineFormatType,
    ) -> Option<DomHandle> {
        if Self::is_format_node(dom.lookup_node(handle), format) {
            Some(handle.clone())
        } else if handle.has_parent() {
            let parent_handle = handle.parent_handle();
            if Self::is_format_node(dom.lookup_node(&parent_handle), format) {
                Some(parent_handle)
            } else {
                Self::path_contains_format_node(dom, &parent_handle, format)
            }
        } else {
            None
        }
    }

    fn is_format_node(node: &DomNode<S>, format: &InlineFormatType) -> bool {
        if let DomNode::Container(n) = node {
            if let ContainerNodeKind::Formatting(kind) = n.kind() {
                if kind == format {
                    return true;
                }
            }
        }
        false
    }

    fn split_text_node_by_offsets(
        loc: &DomLocation,
        node: DomNode<S>,
    ) -> (Option<DomNode<S>>, DomNode<S>, Option<DomNode<S>>) {
        if loc.is_start() {
            let (before, middle) =
                Self::split_text_node(Some(node), loc.start_offset);
            (before, middle.unwrap_or_else(DomNode::new_empty_text), None)
        } else if loc.is_end() {
            let (middle, after) =
                Self::split_text_node(Some(node), loc.end_offset);
            (None, middle.unwrap_or_else(DomNode::new_empty_text), after)
        } else {
            let (before, middle) =
                Self::split_text_node(Some(node), loc.start_offset);
            let (middle, after) = Self::split_text_node(
                middle,
                loc.end_offset - loc.start_offset,
            );
            (
                before,
                middle.unwrap_or_else(DomNode::new_empty_text),
                after,
            )
        }
    }

    fn split_text_node(
        node: Option<DomNode<S>>,
        position: usize,
    ) -> (Option<DomNode<S>>, Option<DomNode<S>>) {
        if let Some(node) = node {
            if let DomNode::Text(text_node) = node {
                if text_node.data().is_empty() {
                    (None, None)
                } else {
                    let split_data_orig =
                        text_node.data()[..position].to_owned();
                    let split_data_new =
                        text_node.data()[position..].to_owned();
                    let before = if split_data_orig.is_empty() {
                        None
                    } else {
                        let mut node = DomNode::new_text(split_data_orig);
                        node.set_handle(text_node.handle());
                        Some(node)
                    };
                    let after = if split_data_new.is_empty() {
                        None
                    } else {
                        Some(DomNode::new_text(split_data_new))
                    };
                    (before, after)
                }
            } else {
                panic!("Node was not a text node so can't be split!");
            }
        } else {
            (None, None)
        }
    }

    /**
     * If the supplied node is a text node with zero length, modify it to
     * contain a zero width space and return 1.
     * Otherwise, return 0 and don't modify anything.
     *
     * Returns the number of characters added, which will either be 0 or 1.
     */
    fn insert_zwspace_if_needed(node: &mut DomNode<S>) -> isize {
        if let DomNode::Text(text) = node {
            if text.data().is_empty() {
                text.set_data("\u{200B}".into());
                1
            } else {
                0
            }
        } else {
            0
        }
    }

    fn merge_formatting_node_with_siblings(
        &mut self,
        handle: &DomHandle,
    ) -> HashMap<DomHandle, DomHandle> {
        // Lists of handles that have been moved by merging nodes
        let mut moved_handles = HashMap::new();
        // If has next sibling, try to join it with the current node
        if let DomNode::Container(parent) =
            self.state.dom.lookup_node(&handle.parent_handle())
        {
            if parent.children().len() - handle.index_in_parent() > 1 {
                self.join_format_node_with_prev(
                    &handle.next_sibling(),
                    &mut moved_handles,
                );
            }
        }
        // Merge current node with previous if possible
        self.join_format_node_with_prev(handle, &mut moved_handles);
        moved_handles
    }
}

#[cfg(test)]
mod test {
    use crate::tests::testutils_composer_model::{cm, tx};

    use super::*;

    fn find_range<S: UnicodeString>(model: &ComposerModel<S>) -> Range {
        let (start, end) = model.get_selection();
        model.state.dom.find_range(start.into(), end.into())
    }

    #[test]
    fn selection_type_extend_if_different_type() {
        let model = cm("{hello <b>wor}|ld</b>");
        let range = find_range(&model);
        let selection_type = model.check_format_selection_type(
            &range.locations,
            &InlineFormatType::Italic,
        );
        assert_eq!(selection_type, FormatSelectionType::Extend);
    }

    #[test]
    fn selection_type_extending_start() {
        let model = cm("hell{o <b>w}|orld</b>");
        let range = find_range(&model);
        let selection_type = model.check_format_selection_type(
            &range.locations,
            &InlineFormatType::Bold,
        );
        assert_eq!(selection_type, FormatSelectionType::Extend);
    }

    #[test]
    fn selection_type_extending_end() {
        let model = cm("<b>hell{o </b>wor}|ld");
        let range = find_range(&model);
        let selection_type = model.check_format_selection_type(
            &range.locations,
            &InlineFormatType::Bold,
        );
        assert_eq!(selection_type, FormatSelectionType::Extend);
    }

    #[test]
    fn selection_type_extending_middle() {
        let model = cm("<b>h{el</b>lo <b>wor}|ld</b>");
        let range = find_range(&model);
        let selection_type = model.check_format_selection_type(
            &range.locations,
            &InlineFormatType::Bold,
        );
        assert_eq!(selection_type, FormatSelectionType::Extend);
    }

    #[test]
    fn selection_type_remove() {
        let model = cm("<b>hel{lo</b><i><b>wor}|ld</b></i>");
        let range = find_range(&model);
        let selection_type = model.check_format_selection_type(
            &range.locations,
            &InlineFormatType::Bold,
        );
        assert_eq!(selection_type, FormatSelectionType::Remove);
    }

    #[test]
    fn selection_type_remove_on_start_edge() {
        let model = cm("{<b>hello </b><i><b>wor}|ld</b></i>");
        let range = find_range(&model);
        let selection_type = model.check_format_selection_type(
            &range.locations,
            &InlineFormatType::Bold,
        );
        assert_eq!(selection_type, FormatSelectionType::Remove);
    }

    #[test]
    fn selection_type_remove_on_ending_edge() {
        let model = cm("<b>hel{lo </b><i><b>world}|</b></i>");
        let range = find_range(&model);
        let selection_type = model.check_format_selection_type(
            &range.locations,
            &InlineFormatType::Bold,
        );
        assert_eq!(selection_type, FormatSelectionType::Remove);
    }

    #[test]
    fn formatting_several_nodes_works_with_different_format() {
        let mut model = cm("{hello <i>wor}|ld</i>");
        model.format(InlineFormatType::Bold);
        assert_eq!(
            model.state.dom.to_string(),
            "<strong>hello\u{A0}</strong><i><strong>wor</strong>ld</i>"
        );
    }

    #[test]
    fn formatting_several_nodes_works_with_same_format() {
        let mut model = cm("{hello <b>wor}|ld</b>");
        model.format(InlineFormatType::Bold);
        assert_eq!(model.state.dom.to_string(), "<strong>hello world</strong>");
    }

    #[test]
    fn formatting_several_nodes_works_with_plain_text_between() {
        let mut model = cm("<b>{hello</b> <b>wor}|ld</b>");
        model.format(InlineFormatType::Bold);
        assert_eq!(model.state.dom.to_string(), "<b>hello world</b>");
    }

    #[test]
    fn formatting_several_nodes_works_with_same_format_rev() {
        let mut model = cm("|{hello <b>wor}ld</b>");
        model.format(InlineFormatType::Bold);
        assert_eq!(model.state.dom.to_string(), "<strong>hello world</strong>");
    }

    #[test]
    fn unformat_across_overlapping_nodes_removes_tag() {
        let mut model = cm("<strong><em>{abc</em>def<em>ghi}|</em></strong>");
        model.unformat(InlineFormatType::Bold);
        assert_eq!(model.state.dom.to_string(), "<em>abc</em>def<em>ghi</em>");
    }

    #[test]
    fn unformat_partial_node_creates_new_formatting_nodes() {
        let mut model = cm("<strong><em>a{bc</em>def<em>gh}|i</em></strong>");
        model.unformat(InlineFormatType::Bold);
        assert_eq!(
            model.state.dom.to_string(),
            "<em><strong>a</strong>bc</em>def<em>gh<strong>i</strong></em>",
        );
    }

    #[test]
    fn unformat_on_edge_creates_new_formatting_node_on_single_side() {
        let mut model = cm("<em>{abc}|def</em>");
        model.unformat(InlineFormatType::Italic);
        assert_eq!(model.state.dom.to_string(), "abc<em>def</em>");

        let mut model = cm("<em>abcd{ef}|</em>");
        model.unformat(InlineFormatType::Italic);
        assert_eq!(model.state.dom.to_string(), "<em>abcd</em>ef");
    }

    #[test]
    fn unformat_across_list_items_removes_tag() {
        let mut model = cm("<ol><li><strong>{abc</strong></li><li><strong>~def}|</strong></li></ol>");
        model.unformat(InlineFormatType::Bold);
        assert_eq!(
            model.state.dom.to_string(),
            "<ol><li>abc</li><li>\u{200b}def</li></ol>"
        );
    }

    #[test]
    fn partially_formatted_selection_triggers_format() {
        let mut model = cm("<em>a{bc</em>de}|f");
        model.italic();
        assert_eq!(model.state.dom.to_string(), "<em>abcde</em>f");
    }

    #[test]
    fn completely_formatted_selection_triggers_unformat() {
        let mut model = cm("<del>a{bcd}|ef</del>");
        model.strike_through();
        assert_eq!(model.state.dom.to_string(), "<del>a</del>bcd<del>ef</del>");
    }

    #[test]
    #[ignore = "Once we re-write the way we handle formatting for empty selection we can restore it"]
    fn format_and_unformat_empty_selection() {
        let mut model = cm("AAA |");
        model.bold();
        assert_eq!(tx(&model), "AAA&nbsp;|");
        model.bold();
        assert_eq!(tx(&model), "AAA&nbsp;|");
    }
}
