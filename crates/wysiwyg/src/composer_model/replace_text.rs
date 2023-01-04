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

use crate::dom::nodes::dom_node::DomNodeKind;
use crate::dom::nodes::dom_node::DomNodeKind::ListItem;
use crate::dom::nodes::DomNode;
use crate::dom::unicode_string::{UnicodeStr, UnicodeStrExt};
use crate::dom::{Dom, DomLocation, Range};
use crate::{
    ComposerModel, ComposerUpdate, DomHandle, Location, UnicodeString,
};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    /// Replaces text in the current selection with new_text.
    /// Treats its input as plain text, so any HTML code will show up in
    /// the document (i.e. it will be escaped).
    pub fn replace_text(&mut self, new_text: S) -> ComposerUpdate<S> {
        self.push_state_to_history();
        self.do_replace_text(new_text)
    }

    /// Replaces text in the an arbitrary start..end range with new_text.
    pub fn replace_text_in(
        &mut self,
        new_text: S,
        start: usize,
        end: usize,
    ) -> ComposerUpdate<S> {
        self.push_state_to_history();
        self.do_replace_text_in(new_text, start, end)
    }

    pub fn enter(&mut self) -> ComposerUpdate<S> {
        self.push_state_to_history();
        self.do_enter()
    }

    fn do_enter(&mut self) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();

        if s == e {
            let range = self.state.dom.find_range(s, e);
            self.enter_with_zero_length_selection(range)
        } else {
            // Clear selection then enter.
            self.do_replace_text_in("".into(), s, e);
            self.do_enter()
        }
    }

    fn enter_with_zero_length_selection(
        &mut self,
        range: Range,
    ) -> ComposerUpdate<S> {
        let position = range.start();
        let leaf_at_cursor: Option<&DomLocation> = range.leaves().find(|loc| {
            loc.position <= position && position <= loc.position + loc.length
        });

        match leaf_at_cursor {
            None => {
                // Selection doesn't contain any text like nodes. We can assume it's an empty Dom.
                self.state
                    .dom
                    .document_mut()
                    .append_child(DomNode::new_line_break());
                self.state.start += 1;
                self.state.end = self.state.start;
                self.create_update_replace_all()
            }
            Some(leaf) => {
                if leaf.length == 0 {
                    // Special case, there is an empty text node at the cursor position
                    self.enter_with_zero_length_selection_and_empty_text_nodes(
                        range,
                    );
                    self.create_update_replace_all()
                } else {
                    let handle = &leaf.node_handle;
                    let ancestor_block_location = range
                        .locations
                        .iter()
                        .filter(|l| {
                            l.kind.is_block_kind()
                                && l.node_handle
                                    .is_ancestor_of(&leaf.node_handle)
                        })
                        .max();

                    if let Some(ancestor_block_location) =
                        ancestor_block_location
                    {
                        self.enter_with_zero_selection_in_block_node(
                            leaf,
                            ancestor_block_location,
                            &range,
                        )
                    } else {
                        self.do_enter_in_text(handle, leaf.start_offset)
                    }
                }
            }
        }
    }

    fn enter_with_zero_selection_in_block_node(
        &mut self,
        leaf: &DomLocation,
        block_ancestor_loc: &DomLocation,
        range: &Range,
    ) -> ComposerUpdate<S> {
        match block_ancestor_loc.kind {
            DomNodeKind::List => {
                // Find ListItem that's an ancestor of the leaf and a child of the List node
                let list_item = range
                    .locations
                    .iter()
                    .find(|l| {
                        block_ancestor_loc
                            .node_handle
                            .is_ancestor_of(&l.node_handle)
                            && l.node_handle.is_ancestor_of(&leaf.node_handle)
                            && l.kind == ListItem
                    })
                    .unwrap();
                let current_cursor_global_location =
                    leaf.position + leaf.start_offset;
                self.do_enter_in_list(
                    &list_item.node_handle,
                    current_cursor_global_location,
                    list_item.end_offset,
                )
            }
            DomNodeKind::CodeBlock => {
                self.do_enter_in_code_block(leaf, block_ancestor_loc)
            }
            DomNodeKind::Quote => {
                self.do_enter_in_quote(leaf, block_ancestor_loc)
            }
            _ => panic!("Enter not implemented for this block node!"),
        }
    }

    fn do_enter_in_code_block(
        &mut self,
        leaf: &DomLocation,
        block_location: &DomLocation,
    ) -> ComposerUpdate<S> {
        // Helper function since we have to repeat this code twice
        fn insert_code_block<S: UnicodeString>(
            dom: &mut Dom<S>,
            sub_tree: DomNode<S>,
            at_handle: &DomHandle,
            new_line_at_end: bool,
        ) {
            dom.insert_at(at_handle, sub_tree);
            let insert_new_line_at = if new_line_at_end {
                at_handle.next_sibling()
            } else {
                at_handle.clone()
            };
            dom.insert_at(&insert_new_line_at, DomNode::new_line_break());
        }

        let mut has_previous_line_break = false;
        let mut selection_offset = 0;
        if leaf.start_offset > 0 {
            if let DomNode::Text(text_node) =
                self.state.dom.lookup_node_mut(&leaf.node_handle)
            {
                let prev_offset = leaf.start_offset - 1;
                let prev_char = text_node.data().chars().nth(prev_offset);
                if let Some(prev_char) = prev_char {
                    if prev_char == '\n' {
                        // Remove line break, we'll add another one outside the code block
                        let mut new_data = text_node.data().to_owned();
                        new_data.remove_at(prev_offset);
                        text_node.set_data(new_data);
                        // Adjust selection too
                        self.state.start -= 1;
                        self.state.end = self.state.start;
                        has_previous_line_break = true;
                        selection_offset += 1;
                    }
                }
            }
        }

        // If there was a previous line break, we need to split the code block and add the line break
        if has_previous_line_break {
            let mut sub_tree = self.state.dom.split_sub_tree_from(
                &leaf.node_handle,
                leaf.start_offset - selection_offset,
                block_location.node_handle.depth(),
            );
            sub_tree.set_handle(DomHandle::root());
            let DomNode::Container(sub_tree_container) = &mut sub_tree else {
                panic!("Sub tree must start from a container node");
            };

            let new_line_at_end = leaf.is_start();

            if !sub_tree_container.is_empty()
                && !sub_tree_container.has_leading_zwsp()
            {
                sub_tree_container.insert_child(0, DomNode::new_zwsp());
            }

            if self.state.dom.contains(&block_location.node_handle) {
                if let DomNode::Container(block) =
                    self.state.dom.lookup_node_mut(&block_location.node_handle)
                {
                    if block.children().is_empty() || block.only_contains_zwsp()
                    {
                        self.state.dom.replace(
                            &block_location.node_handle,
                            vec![
                                DomNode::new_line_break(),
                                DomNode::new_code_block(
                                    sub_tree_container.children().clone(),
                                ),
                            ],
                        );
                    } else {
                        let next_handle =
                            block_location.node_handle.next_sibling();
                        if !sub_tree_container.is_empty()
                            && !sub_tree_container.only_contains_zwsp()
                        {
                            self.state.dom.insert_at(
                                &next_handle,
                                DomNode::new_code_block(
                                    sub_tree_container.children().clone(),
                                ),
                            );
                        }
                        self.state
                            .dom
                            .insert_at(&next_handle, DomNode::new_line_break());
                    }
                } else {
                    // Insert the whole code block at its old DomHandle
                    insert_code_block(
                        &mut self.state.dom,
                        sub_tree,
                        &block_location.node_handle,
                        new_line_at_end,
                    );
                }
            } else {
                // Insert the whole code block at its old DomHandle
                insert_code_block(
                    &mut self.state.dom,
                    sub_tree,
                    &block_location.node_handle,
                    new_line_at_end,
                );
            }
            // Fix selection if it was after the first possible line break
            if block_location.start_offset > 2 {
                self.state.start += 1;
                self.state.end = self.state.start;
            }
            self.create_update_replace_all()
        } else {
            // Otherwise, just add a ('\n') line break
            let DomNode::Container(block) = self.state.dom.lookup_node(&block_location.node_handle) else {
                panic!("Parent must be a block node");
            };
            if block.children().is_empty() {
                // If it's the first character, we need to add 2 line breaks instead, since the
                // first one will be automatically removed by any HTML parser
                self.state.dom.insert_at(
                    &leaf.node_handle,
                    DomNode::new_text("\n\n".into()),
                );
                self.state.start += 2;
                self.state.end = self.state.start;
                self.create_update_replace_all()
            } else if let DomNode::Text(text_node) =
                self.state.dom.lookup_node_mut(&leaf.node_handle)
            {
                // Just add the line break and move the selection
                let mut new_data = text_node.data().to_owned();
                new_data.insert(leaf.start_offset, &S::from("\n"));
                text_node.set_data(new_data);
                self.state.start += 1;
                self.state.end = self.state.start;
                self.create_update_replace_all()
            } else if let DomNode::Zwsp(_) =
                self.state.dom.lookup_node(&leaf.node_handle)
            {
                let text_node = DomNode::new_text("\n".into());
                self.state
                    .dom
                    .insert_at(&leaf.node_handle.next_sibling(), text_node);
                self.state.start += 1;
                self.state.end = self.state.start;
                self.create_update_replace_all()
            } else {
                ComposerUpdate::keep()
            }
        }
    }

    fn do_enter_in_quote(
        &mut self,
        leaf: &DomLocation,
        block_location: &DomLocation,
    ) -> ComposerUpdate<S> {
        let has_previous_line_break = leaf.kind == DomNodeKind::LineBreak;

        if has_previous_line_break {
            // If there was a previous line break we should split the quote in 2 parts, adding a
            // line break between them.
            let mut sub_tree = self.state.dom.split_sub_tree_from(
                &leaf.node_handle,
                leaf.start_offset,
                block_location.node_handle.depth(),
            );
            // Needed to be able to add children
            sub_tree.set_handle(DomHandle::root());
            let DomNode::Container(sub_tree_container) = &mut sub_tree else {
                panic!("Sub tree must start from a container node");
            };

            let insert_at =
                if self.state.dom.contains(&block_location.node_handle) {
                    if block_location.start_offset > 0 {
                        block_location.node_handle.next_sibling()
                    } else {
                        block_location.node_handle.clone()
                    }
                } else {
                    block_location.node_handle.clone()
                };

            // We don't need the leading line break in the extracted half of the block quote
            if sub_tree_container.remove_leading_line_break() {
                self.state.start -= 1;
                self.state.end -= 1;
            }

            if !sub_tree_container.is_empty()
                && !sub_tree_container.only_contains_zwsp()
            {
                // If the sub-tree is not empty, insert it along with a line break before it
                if !sub_tree_container.has_leading_zwsp() {
                    sub_tree_container.insert_child(0, DomNode::new_zwsp());
                    // Don't modify selection if we're inserting it at the same position as the old block
                    if insert_at != block_location.node_handle {
                        self.state.start += 1;
                        self.state.end += 1;
                    }
                }
                self.state.dom.insert_at(
                    &insert_at,
                    DomNode::new_quote(sub_tree_container.children().clone()),
                );
                self.state
                    .dom
                    .insert_at(&insert_at, DomNode::new_line_break());
            } else {
                // If the sub-tree is empty, just add the line break
                let insert_at =
                    if self.state.dom.contains(&block_location.node_handle) {
                        self.state.start += 1;
                        self.state.end += 1;
                        block_location.node_handle.next_sibling()
                    } else {
                        block_location.node_handle.clone()
                    };
                self.state
                    .dom
                    .insert_at(&insert_at, DomNode::new_line_break());
            }
        } else {
            self.do_enter_in_text(&leaf.node_handle, leaf.start_offset);
        }

        self.create_update_replace_all()
    }

    fn enter_with_zero_length_selection_and_empty_text_nodes(
        &mut self,
        range: Range,
    ) {
        let leaves = range.leaves();
        let empty_text_leaves: Vec<&DomLocation> = leaves
            .into_iter()
            .filter(|l| {
                if let DomNode::Text(t) =
                    self.state.dom.lookup_node(&l.node_handle)
                {
                    t.data().is_empty()
                } else {
                    false
                }
            })
            .collect();
        for (i, leaf) in empty_text_leaves.iter().enumerate().rev() {
            if i == 0 {
                self.state.dom.replace(
                    &leaf.node_handle,
                    vec![DomNode::new_line_break()],
                );
            } else {
                self.state.dom.remove(&leaf.node_handle);
            }
        }
        self.state.start += 1;
        self.state.end = self.state.start;
    }

    pub(crate) fn do_replace_text(&mut self, new_text: S) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();
        self.do_replace_text_in(new_text, s, e)
    }

    /// Internal: replace some text without modifying the undo/redo state.
    pub(crate) fn do_replace_text_in(
        &mut self,
        new_text: S,
        start: usize,
        end: usize,
    ) -> ComposerUpdate<S> {
        let text_string = new_text.to_string();
        // If the inserted text contains newlines, slice it and
        // insert each slice while simulating calls to the
        // enter function in betweeen.
        if text_string.contains('\n') {
            let mut slices = text_string.split('\n').peekable();
            while let Some(slice) = slices.next() {
                let (s, e) = self.safe_selection();
                self.do_replace_text_in(S::from(slice), s, e);
                if slices.peek().is_some() {
                    self.do_enter();
                }
            }
        } else {
            let len = new_text.len();
            self.state.dom.replace_text_in(new_text, start, end);
            self.apply_pending_formats(start, start + len);
            self.state.start = Location::from(start + len);
            self.state.end = self.state.start;
        }

        // TODO: for now, we replace every time, to check ourselves, but
        // at least some of the time we should not
        self.create_update_replace_all()
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use widestring::Utf16String;

    use crate::action_state::ActionState;
    use crate::menu_state::MenuStateUpdate;
    use crate::tests::testutils_composer_model::cm;
    use crate::tests::testutils_conversion::utf16;
    use crate::{ComposerAction, ComposerUpdate, Location, MenuState};
    use strum::IntoEnumIterator;

    #[test]
    fn composer_update_contains_escaped_html() {
        let mut model = cm("|");
        let update = model.replace_text(Utf16String::from_str("<"));
        assert_eq!(
            update,
            ComposerUpdate::replace_all(
                utf16("&lt;"),
                Location::from(1),
                Location::from(1),
                MenuState::Update(MenuStateUpdate {
                    action_states: indent_unindent_redo_disabled()
                }),
            )
        );
    }

    fn indent_unindent_redo_disabled() -> HashMap<ComposerAction, ActionState> {
        let actions = ComposerAction::iter().map(|action| {
            if matches!(
                action,
                ComposerAction::Redo
                    | ComposerAction::Indent
                    | ComposerAction::UnIndent
            ) {
                (action, ActionState::Disabled)
            } else {
                (action, ActionState::Enabled)
            }
        });
        HashMap::from_iter(actions)
    }
}
