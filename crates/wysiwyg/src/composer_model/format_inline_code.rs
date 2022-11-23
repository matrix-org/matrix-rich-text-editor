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

use crate::composer_model::menu_state::MenuStateComputeType;
use crate::dom::nodes::{DomNode, TextNode};
use crate::dom::unicode_string::UnicodeStrExt;
use crate::dom::{DomHandle, DomLocation};
use crate::{ComposerModel, ComposerUpdate, InlineFormatType, UnicodeString};

/// Special implementations of formatting for inline code
impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub(crate) fn add_inline_code(&mut self) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();
        let format = InlineFormatType::InlineCode;

        if s == e {
            self.toggle_zero_length_format(&format);
            ComposerUpdate::update_menu_state(
                self.compute_menu_state(MenuStateComputeType::KeepIfUnchanged),
            )
        } else {
            let range = self.state.dom.find_range(s, e);
            let leaves: Vec<&DomLocation> = range.leaves().collect();
            // We'll iterate through the leaves finding their closest structural node ancestor and
            // grouping these leaves based on the handles of these ancestors.
            let structure_ancestors = self
                .group_leaves_by_closest_structure_ancestors(leaves.clone());

            // Order those ancestors (important to avoid node replacement & conflicts of handles)
            let mut keys: Vec<&DomHandle> =
                structure_ancestors.keys().collect();
            keys.sort();

            // Iterate through them backwards, replacing their descendant leaves as needed
            for ancestor_handle in keys.into_iter().rev() {
                let leaves = structure_ancestors.get(ancestor_handle).unwrap();
                // We'll store the text contents of the removed formatted nodes here
                let mut cur_text = S::default();
                // Where we'll insert the result of merging the text contents
                let mut insert_text_at: Option<DomHandle> = None;
                // Nodes to be added to the Dom, might contain both TextNodes and LineBreaks
                let mut nodes_to_add = Vec::new();
                // Iterate the leaves backwards to avoid modifying the previous Dom structure
                for leaf in leaves.into_iter().rev() {
                    // Find the immediate child of the common ancestor containing this leaf as its descendant
                    let ancestor_child_handle = leaf
                        .node_handle
                        .sub_handle_up_to(ancestor_handle.raw().len() + 1);

                    let node =
                        self.state.dom.lookup_node(&leaf.node_handle).clone();
                    match node {
                        DomNode::Text(text_node) => {
                            let (text, pos) = self
                                .process_text_node_for_inline_code(
                                    &text_node,
                                    leaf,
                                    &ancestor_child_handle,
                                );
                            // Add the selected text to the current text holder
                            cur_text.insert(0, &text);
                            // Update insertion position for the inline code node
                            insert_text_at = pos;
                        }
                        DomNode::LineBreak(_) => {
                            nodes_to_add.extend(
                                self.process_line_break_for_inline_code(
                                    &leaf, &cur_text,
                                ),
                            );
                            // Update insertion point and reset text
                            insert_text_at = Some(ancestor_child_handle);
                            cur_text = S::default();
                        }
                        _ => panic!(
                            "Leaf should be either a line break or a text node"
                        ),
                    }
                }

                // Insert the nodes into the Dom inside an inline code node
                if insert_text_at.is_some() {
                    let insert_at = insert_text_at.unwrap();

                    // If there is still some collected text add it to he list of nodes to insert
                    if !cur_text.is_empty() {
                        nodes_to_add.insert(0, DomNode::new_text(cur_text));
                    }

                    // Insert the inline code node
                    self.state.dom.insert_at(
                        &insert_at,
                        DomNode::new_formatting(
                            InlineFormatType::InlineCode,
                            nodes_to_add,
                        ),
                    );

                    // Merge inline code nodes for clean up
                    self.merge_formatting_node_with_siblings(&insert_at);
                }
            }
            self.create_update_replace_all()
        }
    }

    fn process_text_node_for_inline_code(
        &mut self,
        text_node: &TextNode<S>,
        location: &DomLocation,
        ancestor_child_handle: &DomHandle,
    ) -> (S, Option<DomHandle>) {
        let mut insert_text_at = None;
        // Get the selected text from the TextNode
        let text = text_node.data()[location.start_offset..location.end_offset]
            .to_owned();
        let handle = &location.node_handle;
        let dom = &mut self.state.dom;

        if location.is_covered() {
            // This node is covered, remove it and any empty ancestors and set
            // the insertion point to be at its position.
            insert_text_at = Some(ancestor_child_handle.clone());
            self.remove_and_clean_up_empty_nodes_until(
                handle,
                ancestor_child_handle,
            );
        } else if location.is_start() {
            // This node is at the start of the selection and not completely
            // covered, split it and set the insertion point to be after it.
            insert_text_at = Some(ancestor_child_handle.next_sibling());
            let text = text_node.data()[..location.start_offset].to_owned();
            dom.replace(handle, vec![DomNode::new_text(text)]);
        } else if location.is_end() {
            // This node is at the end of the selection and not completely
            // covered, split it and set the insertion point to be before it.
            insert_text_at = if ancestor_child_handle.index_in_parent() > 0 {
                Some(ancestor_child_handle.prev_sibling())
            } else {
                Some(ancestor_child_handle.clone())
            };
            let text = text_node.data()[location.end_offset..].to_owned();
            dom.replace(handle, vec![DomNode::new_text(text)]);
        }

        (text, insert_text_at)
    }

    fn process_line_break_for_inline_code(
        &mut self,
        location: &DomLocation,
        cur_text: &S,
    ) -> Vec<DomNode<S>> {
        let mut nodes_to_add = Vec::new();
        // Get any pending text and create a new TextNode to insert along with
        // the LineBreak one, removing the old LineBreak node.
        if !cur_text.is_empty() {
            nodes_to_add.insert(0, DomNode::new_text(cur_text.clone()));
        }
        nodes_to_add.insert(0, DomNode::new_line_break());
        self.state.dom.remove(&location.node_handle);
        nodes_to_add
    }
}

#[cfg(test)]
mod test {
    use crate::tests::testutils_composer_model::{cm, tx};

    #[test]
    fn inline_code_replacing_formatting_removes_formatting() {
        let mut model = cm("<b>{bold</b><i>text}|</i>");
        model.inline_code();
        assert_eq!(tx(&model), "<code>{boldtext}|</code>");
    }

    #[test]
    fn inline_code_replacing_partial_formatting_removes_overlapping_formatting()
    {
        let mut model = cm("<b>bo{ld</b><i>te}|xt</i>");
        model.inline_code();
        assert_eq!(tx(&model), "<b>bo</b><code>{ldte}|</code><i>xt</i>");
    }

    #[test]
    fn inline_code_with_formatting_preserves_line_breaks() {
        let mut model = cm("<b>{bold</b><br /><i>text}|</i>");
        model.inline_code();
        assert_eq!(tx(&model), "<code>{bold<br />text}|</code>");
    }

    #[test]
    fn inline_code_replacing_complex_formatting_removes_formatting() {
        let mut model = cm("<b><u>{bold</u></b><i>text}|</i>");
        model.inline_code();
        assert_eq!(tx(&model), "<code>{boldtext}|</code>");
    }

    #[test]
    fn inline_code_replacing_nested_and_complex_formatting_removes_formatting()
    {
        let mut model = cm("<b><u>{bold</u><i>italic</i></b><i>text}|</i>");
        model.inline_code();
        assert_eq!(tx(&model), "<code>{bolditalictext}|</code>");
    }

    #[test]
    fn inline_code_partially_replacing_formatting_removes_overlap() {
        let mut model = cm("<b><u>bo{ld</u></b><i>te}|xt</i>");
        model.inline_code();
        assert_eq!(tx(&model), "<b><u>bo</u></b><code>{ldte}|</code><i>xt</i>");
    }

    #[test]
    fn inline_code_on_partial_nested_line_break_removes_formatting() {
        let mut model = cm("<b><u>bo{ld</u></b><br /><i>te}|xt</i>");
        model.inline_code();
        assert_eq!(
            tx(&model),
            "<b><u>bo</u></b><code>{ld<br />te}|</code><i>xt</i>"
        );
    }

    #[test]
    fn inline_code_on_partial_nested_line_break_within_parent_removes_formatting(
    ) {
        let mut model = cm("<b><u>bo{ld</u><br /></b><i>te}|xt</i>");
        model.inline_code();
        assert_eq!(
            tx(&model),
            "<b><u>bo</u></b><code>{ld<br />te}|</code><i>xt</i>"
        );
    }

    #[test]
    fn format_inline_code_in_list_item() {
        let mut model = cm("<ul><li><b>bo{ld</b><i>text}|</i></li></ul>");
        model.inline_code();
        assert_eq!(
            tx(&model),
            "<ul><li><b>bo</b><code>{ldtext}|</code></li></ul>"
        );
    }

    #[test]
    fn format_inline_code_in_several_list_items() {
        let mut model =
            cm("<ul><li><b>bo{ld</b></li><li><i>text}|</i></li></ul>");
        model.inline_code();
        assert_eq!(
            tx(&model),
            "<ul><li><b>bo</b><code>{ld</code></li><li><code>text}|</code></li></ul>"
        );
    }

    #[test]
    fn format_inline_code_in_several_list_items_and_text() {
        let mut model =
            cm("Text {before<br /><ul><li><b>bo}|ld</b></li><li><i>text</i></li></ul>");
        model.inline_code();
        assert_eq!(
            tx(&model),
            "Text <code>{before<br /></code><ul><li><code>bo}|</code><b>ld</b></li><li><i>text</i></li></ul>"
        );
    }

    #[test]
    fn format_inline_code_with_existing_inline_code_start() {
        let mut model = cm("{Some <code>co}|de</code>");
        model.inline_code();
        assert_eq!(tx(&model), "<code>{Some co}|de</code>");
    }

    #[test]
    fn format_inline_code_with_existing_inline_code_end() {
        let mut model = cm("<code>So{me </code>code}|");
        model.inline_code();
        assert_eq!(tx(&model), "<code>So{me code}|</code>");
    }

    #[test]
    fn format_inline_code_with_existing_inline_code_side_to_side_start() {
        let mut model = cm("<code>Some </code>{code}|");
        model.inline_code();
        assert_eq!(tx(&model), "<code>Some {code}|</code>");
    }

    #[test]
    fn format_inline_code_with_existing_inline_code_side_to_side_end() {
        let mut model = cm("{Some }|<code>code</code>");
        model.inline_code();
        assert_eq!(tx(&model), "<code>{Some }|code</code>");
    }

    #[test]
    fn unformat_inline_code_same_row_with_line_breaks() {
        let mut model = cm("<code>{bold<br />text}|</code>");
        model.inline_code();
        assert_eq!(tx(&model), "{bold<br />text}|");
    }

    #[test]
    fn unformat_inline_code_in_several_list_items_and_text() {
        let mut model =
            cm("Text <code>{before<br /></code><ul><li><code>bo}|</code><b>ld</b></li><li><i>text</i></li></ul>");
        model.inline_code();
        assert_eq!(
            tx(&model),
            "Text {before<br /><ul><li>bo}|<b>ld</b></li><li><i>text</i></li></ul>"
        );
    }

    // TODO: might need to re-visit it if we add ZWSP at the end of inline code tags,
    // otherwise this test should actually follow the same behaviour as those in `test_formatting.rs`
    // for 'unformatting_...'.
    #[test]
    fn disable_inline_code_then_write_text() {
        let mut model = cm("<code>code|</code>");
        model.inline_code();
        model.replace_text(" plain text".into());
        assert_eq!(tx(&model), "<code>code</code> plain text|");
    }
}
