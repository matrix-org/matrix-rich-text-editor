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

use crate::composer_model::base::{slice, slice_from, slice_to};
use crate::dom::nodes::{ContainerNodeKind, DomNode, TextNode};
use crate::dom::{
    Dom, DomHandle, DomLocation, MultipleNodesRange, Range, SameNodeRange,
};
use crate::{
    ComposerAction, ComposerModel, ComposerUpdate, InlineFormatType, Location,
    UnicodeString,
};

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
        match range {
            Range::SameNode(range) => {
                self.format_same_node(range, format);
            }

            Range::NoNode => {
                self.state.dom.append_child(DomNode::new_formatting(
                    format,
                    vec![DomNode::new_text(S::from_str(""))],
                ));
            }

            Range::MultipleNodes(range) => {
                self.format_several_nodes(&range, format);
            }
        }
    }

    pub fn unformat(&mut self, format: InlineFormatType) -> ComposerUpdate<S> {
        // Store current Dom
        self.push_state_to_history();
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        match range {
            Range::SameNode(range) => {
                self.unformat_same_node(s, e, range, format);
                self.create_update_replace_all()
            }

            Range::MultipleNodes(range) => {
                self.unformat_several_nodes(s, e, &range, format);
                self.create_update_replace_all()
            }

            Range::NoNode => {
                panic!("Trying to unformat with no selected node")
            }
        }
    }

    fn format_same_node(
        &mut self,
        range: SameNodeRange,
        format: InlineFormatType,
    ) {
        let node = self.state.dom.lookup_node(&range.node_handle);
        if let DomNode::Text(t) = node {
            let text = t.data();
            // TODO: can we be globally smart about not leaving empty text nodes ?
            let before = slice_to(text, ..range.start_offset);
            let during = slice(text, range.start_offset..range.end_offset);
            let after = slice_from(text, range.end_offset..);
            let mut new_nodes = Vec::new();
            if before.len() > 0 {
                new_nodes.push(DomNode::new_text(before));
            }
            new_nodes.push(DomNode::new_formatting(
                format,
                vec![DomNode::new_text(during)],
            ));
            if after.len() > 0 {
                new_nodes.push(DomNode::new_text(after));
            }
            self.state.dom.replace(&range.node_handle, new_nodes);
        } else {
            panic!("Trying to format a non-text node")
        }
    }

    fn unformat_same_node(
        &mut self,
        start: usize,
        end: usize,
        range: SameNodeRange,
        format: InlineFormatType,
    ) {
        let text_node = self.state.dom.lookup_node(&range.node_handle);
        if let DomNode::Text(t) = text_node {
            let text_length = t.data().len().clone();
            let formatting_handle = self.find_parent_formatting_node(
                range.node_handle.clone(),
                format.clone(),
            );
            let formatting_node =
                self.state.dom.lookup_node(&formatting_handle);
            if formatting_node.is_container_node() {
                self.state.dom.remove_and_keep_children(&formatting_handle);
            } else {
                panic!("Mismatched type for formatting container")
            }

            let (sb, eb) = self.safe_locations_from(
                Location::from(start - range.start_offset),
                Location::from(start),
            );
            let (sa, ea) = self.safe_locations_from(
                Location::from(end),
                Location::from(end + text_length - range.end_offset),
            );

            // Re-apply formatting to slices before and after the selection if needed
            if eb > sb {
                self.format_range(sb, eb, format.clone());
            }
            if ea > sa {
                self.format_range(sa, ea, format);
            }
        } else {
            panic!("Mismatched type for text node")
        }
    }

    fn check_format_selection_type(
        &self,
        locations: &Vec<DomLocation>,
        format: &InlineFormatType,
    ) -> FormatSelectionType {
        // First sweep to understand what the underlying DOM looks like
        let found_format_locations: Vec<&DomLocation> = locations
            .iter()
            .filter(|l| {
                let node = self.state.dom.lookup_node(&l.node_handle);
                Self::is_format_node(node, format)
            })
            .collect();

        // No format nodes found, so it can we can only create new formatting nodes by Extend
        if found_format_locations.is_empty() {
            FormatSelectionType::Extend
        } else {
            // Find text nodes inside the selection that are not formatted with this format
            let non_formatted_leaf_locations = locations.iter().filter(|l| {
                l.is_leaf
                    && Self::path_contains_format_node(
                        &self.state.dom,
                        &l.node_handle,
                        &format,
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
        range: &MultipleNodesRange,
        format: InlineFormatType,
    ) {
        let selection_type =
            self.check_format_selection_type(&range.locations, &format);
        match selection_type {
            FormatSelectionType::Remove => {} // TODO: actually implement this
            FormatSelectionType::Extend => self
                .extend_format_in_multiple_nodes(
                    range.locations.clone(),
                    &format,
                ),
        }
    }

    fn unformat_several_nodes(
        &mut self,
        start: usize,
        end: usize,
        range: &MultipleNodesRange,
        format: InlineFormatType,
    ) {
        for location in range.locations.iter() {
            let node = self.state.dom.lookup_node(&location.node_handle);
            if let DomNode::Container(node) = node {
                if let ContainerNodeKind::Formatting(f) = node.kind() {
                    if *f == format {
                        self.state
                            .dom
                            .remove_and_keep_children(&location.node_handle);

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

    fn find_parent_formatting_node(
        &self,
        handle: DomHandle,
        format: InlineFormatType,
    ) -> DomHandle {
        let node = self.state.dom.lookup_node(&handle);
        if let DomNode::Container(container) = node {
            match container.kind() {
                ContainerNodeKind::Formatting(f) => {
                    if *f == format {
                        handle
                    } else {
                        self.find_parent_formatting_node(
                            handle.parent_handle(),
                            format,
                        )
                    }
                }
                _ => self.find_parent_formatting_node(
                    handle.parent_handle(),
                    format,
                ),
            }
        } else {
            self.find_parent_formatting_node(handle.parent_handle(), format)
        }
    }

    fn needs_format(
        dom: &Dom<S>,
        loc: &DomLocation,
        format: &InlineFormatType,
    ) -> bool {
        loc.is_leaf
            && Self::path_contains_format_node(dom, &loc.node_handle, format)
                .is_none()
    }

    fn extend_format_in_multiple_nodes(
        &mut self,
        locations: Vec<DomLocation>,
        format: &InlineFormatType,
    ) {
        // Go through the locations in reverse order to prevent Dom modification issues
        for loc in locations.iter().rev() {
            if Self::needs_format(&self.state.dom, loc, &format) {
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
                        // Node only partially covered by selection, we need to split the text node,
                        // add one part to a new formatting node, then replace the original text
                        // node with both the new formatting node and the other half of the text
                        // node to their original parent.
                        let position = if loc.is_start() {
                            loc.start_offset
                        } else {
                            loc.end_offset
                        };
                        if let Some((orig, new)) =
                            Self::split_text_node(node, position)
                        {
                            if loc.is_start() {
                                let new = DomNode::new_formatting(
                                    format.clone(),
                                    vec![DomNode::Text(new)],
                                );
                                parent.insert_child(index, new);
                                parent.insert_child(index, DomNode::Text(orig));
                            } else {
                                let orig = DomNode::new_formatting(
                                    format.clone(),
                                    vec![DomNode::Text(orig)],
                                );
                                parent.insert_child(index, DomNode::Text(new));
                                parent.insert_child(index, orig);
                            }
                        } else {
                            panic!("Node was not a text node so it cannot be split.");
                        }
                    }
                    // Clean up by removing any empty text nodes and merging formatting nodes
                    self.merge_formatting_node_with_siblings(&loc.node_handle);
                }
            }
        }
    }

    fn path_contains_format_node(
        dom: &Dom<S>,
        handle: &DomHandle,
        format: &InlineFormatType,
    ) -> Option<DomHandle> {
        if Self::is_format_node(dom.lookup_node(&handle), format) {
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

    fn split_text_node(
        node: DomNode<S>,
        position: usize,
    ) -> Option<(TextNode<S>, TextNode<S>)> {
        if let DomNode::Text(text_node) = node {
            let split_data_orig = slice_to(text_node.data(), ..position);
            let split_data_new = slice_from(text_node.data(), position..);
            let mut orig = TextNode::from(split_data_orig);
            orig.set_handle(text_node.handle());
            let new = TextNode::from(split_data_new);
            Some((orig, new))
        } else {
            None
        }
    }

    fn merge_formatting_node_with_siblings(&mut self, handle: &DomHandle) {
        // If has next sibling, try to join it with the current node
        if let DomNode::Container(parent) =
            self.state.dom.lookup_node(&handle.parent_handle())
        {
            if parent.children().len() - handle.index_in_parent() > 1 {
                self.join_format_node_with_prev(&handle.next_sibling());
            }
        }
        // Merge current node with previous if possible
        self.join_format_node_with_prev(handle);
    }
}

#[cfg(test)]
mod test {
    use crate::tests::testutils_composer_model::cm;

    use super::*;

    fn find_range<S: UnicodeString>(model: &ComposerModel<S>) -> Range {
        let (start, end) = model.get_selection();
        model.state.dom.find_range(start.into(), end.into())
    }

    fn find_multiple_range<S: UnicodeString>(
        model: &ComposerModel<S>,
    ) -> MultipleNodesRange {
        let range = find_range(&model);
        if let Range::MultipleNodes(r) = range {
            r
        } else {
            panic!("Should have been a multiple nodes range, {:?}", range);
        }
    }

    #[test]
    fn selection_type_extend_if_different_type() {
        let model = cm("{hello <b>wor}|ld</b>");
        let range = find_multiple_range(&model);
        let selection_type = model.check_format_selection_type(
            &range.locations,
            &InlineFormatType::Italic,
        );
        assert_eq!(selection_type, FormatSelectionType::Extend);
    }

    #[test]
    fn selection_type_extending_start() {
        let model = cm("hell{o <b>w}|orld</b>");
        let range = find_multiple_range(&model);
        let selection_type = model.check_format_selection_type(
            &range.locations,
            &InlineFormatType::Bold,
        );
        assert_eq!(selection_type, FormatSelectionType::Extend);
    }

    #[test]
    fn selection_type_extending_end() {
        let model = cm("<b>hell{o </b>wor}|ld");
        let range = find_multiple_range(&model);
        let selection_type = model.check_format_selection_type(
            &range.locations,
            &InlineFormatType::Bold,
        );
        assert_eq!(selection_type, FormatSelectionType::Extend);
    }

    #[test]
    fn selection_type_extending_middle() {
        let model = cm("<b>h{el</b>lo <b>wor}|ld</b>");
        let range = find_multiple_range(&model);
        let selection_type = model.check_format_selection_type(
            &range.locations,
            &InlineFormatType::Bold,
        );
        assert_eq!(selection_type, FormatSelectionType::Extend);
    }

    #[test]
    fn selection_type_remove() {
        let model = cm("<b>hel{lo</b><i><b>wor}|ld</b></i>");
        let range = find_multiple_range(&model);
        let selection_type = model.check_format_selection_type(
            &range.locations,
            &InlineFormatType::Bold,
        );
        assert_eq!(selection_type, FormatSelectionType::Remove);
    }

    #[test]
    fn selection_type_remove_on_start_edge() {
        let model = cm("{<b>hello </b><i><b>wor}|ld</b></i>");
        let range = find_multiple_range(&model);
        let selection_type = model.check_format_selection_type(
            &range.locations,
            &InlineFormatType::Bold,
        );
        assert_eq!(selection_type, FormatSelectionType::Remove);
    }

    #[test]
    fn selection_type_remove_on_ending_edge() {
        let model = cm("<b>hel{lo </b><i><b>world}|</b></i>");
        let range = find_multiple_range(&model);
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
            "<strong>hello </strong><i><strong>wor</strong>ld</i>"
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
}
