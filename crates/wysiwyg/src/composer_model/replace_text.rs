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

use crate::dom::action_list::{DomAction, DomActionList};
use crate::dom::nodes::dom_node::DomNodeKind::{Link, ListItem};
use crate::dom::nodes::{DomNode, TextNode};
use crate::dom::unicode_string::{UnicodeStrExt, UnicodeStringExt};
use crate::dom::{DomHandle, DomLocation, Range};
use crate::{ComposerModel, ComposerUpdate, Location, UnicodeString};

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
        let leaves: Vec<&DomLocation> = range.leaves().collect();
        if leaves.len() == 1 {
            let location = leaves[0];
            let current_cursor_global_location =
                location.position + location.start_offset;
            let handle = &location.node_handle;
            let parent_list_item_handle =
                self.state.dom.find_parent_list_item_or_self(handle);
            if let Some(parent_list_item_handle) = parent_list_item_handle {
                let list_item_end_offset = range
                    .locations
                    .into_iter()
                    .filter(|loc| loc.kind == ListItem)
                    .next()
                    .unwrap()
                    .end_offset;
                self.do_enter_in_list(
                    &parent_list_item_handle,
                    current_cursor_global_location,
                    list_item_end_offset,
                )
            } else {
                self.do_enter_in_text(handle, location.start_offset)
            }
        } else if leaves.is_empty() {
            // Selection doesn't contain any text node. We can assume it's an empty Dom.
            self.state
                .dom
                .document_mut()
                .append_child(DomNode::new_line_break());
            self.state.start += 1;
            self.state.end = self.state.start;
            self.create_update_replace_all()
        } else {
            // Special case, there might be one or several empty text nodes at the cursor position
            self.enter_with_zero_length_selection_and_empty_text_nodes(leaves);
            self.create_update_replace_all()
        }
    }

    fn enter_with_zero_length_selection_and_empty_text_nodes(
        &mut self,
        leaves: Vec<&DomLocation>,
    ) {
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
        mut start: usize,
        end: usize,
    ) -> ComposerUpdate<S> {
        self.state.dom.explicitly_assert_invariants();

        let text_string = new_text.to_string();
        // If the inserted text contains newlines, slice it and
        // insert each slice while simulating calls to the
        // enter function in betweeen.
        if text_string.contains("\n") {
            let mut slices = text_string.split("\n").peekable();
            while let Some(slice) = slices.next() {
                let (s, e) = self.safe_selection();
                self.do_replace_text_in(S::from(slice), s, e);
                if !slices.peek().is_none() {
                    self.do_enter();
                }
            }
        } else {
            let len = new_text.len();
            let range = self.state.dom.find_range(start, end);
            let deleted_handles = if range.is_empty() {
                if !new_text.is_empty() {
                    self.state
                        .dom
                        .append_at_end_of_document(DomNode::new_text(new_text));
                }
                start = 0;
                Vec::new()
            // We check for the first starting_link_handle if any
            // Because for links we always add the text to the next sibling
            } else if let Some(starting_link_handle) =
                self.first_shrinkable_link_node_handle(&range)
            {
                // We replace and delete as normal with an empty string on the current range
                let deleted_handles =
                    self.replace_multiple_nodes(&range, "".into());
                // Then we set the new text value in the next sibling node (or create a new one if none exists)
                self.set_new_text_in_next_sibling_node(
                    starting_link_handle,
                    new_text,
                );
                deleted_handles
            } else {
                self.replace_multiple_nodes(&range, new_text)
            };

            self.apply_pending_formats(start, start + len);

            self.merge_adjacent_text_nodes_after_replace(
                range,
                deleted_handles,
            );

            self.state.start = Location::from(start + len);
            self.state.end = self.state.start;
        }
        self.state.dom.explicitly_assert_invariants();

        // TODO: for now, we replace every time, to check ourselves, but
        // at least some of the time we should not
        self.create_update_replace_all()
    }

    fn merge_adjacent_text_nodes_after_replace(
        &mut self,
        replaced_range: Range,
        deleted_handles: Vec<DomHandle>,
    ) {
        // If we've ended up with adjacent text nodes, merge them
        if let Some(first_location) = replaced_range.locations.first() {
            let first_handle = &first_location.node_handle;
            if deleted_handles.contains(first_handle) {
                // If we deleted the first node in the range ...
                if first_handle.index_in_parent() > 0 {
                    // ... and that was not the first in its parent,
                    // then merge the node before with the next.
                    let prev_handle = first_handle.prev_sibling();
                    self.state.dom.merge_text_nodes_around(&prev_handle);
                }
            } else {
                // If the first node of the range still exists, then
                // merge it with the next, and potentially also the
                // previous.
                self.state
                    .dom
                    .merge_text_nodes_around(&first_location.node_handle);
            }
        }
    }

    fn set_new_text_in_next_sibling_node(
        &mut self,
        node_handle: DomHandle,
        new_text: S,
    ) {
        if let Some(sibling_text_node) =
            self.first_next_sibling_text_node_mut(&node_handle)
        {
            let mut data = sibling_text_node.data().to_owned();
            data.insert(0, &new_text);
            sibling_text_node.set_data(data);
        } else if !new_text.is_empty() {
            let new_child = DomNode::new_text(new_text);
            let parent = self.state.dom.parent_mut(&node_handle);
            let index = node_handle.index_in_parent() + 1;
            parent.insert_child(index, new_child);
        }
    }

    fn first_next_sibling_text_node_mut(
        &mut self,
        node_handle: &DomHandle,
    ) -> Option<&mut TextNode<S>> {
        let parent = self.state.dom.parent(node_handle);
        let children_number = parent.children().len();
        if node_handle.index_in_parent() < children_number - 1 {
            let sibling =
                self.state.dom.lookup_node_mut(&node_handle.next_sibling());
            let DomNode::Text(sibling_text_node) = sibling else {
                return None
            };
            Some(sibling_text_node)
        } else {
            None
        }
    }

    fn first_shrinkable_link_node_handle(
        &self,
        range: &Range,
    ) -> Option<DomHandle> {
        let Some(link_loc) = range.locations.iter().find(|loc| {
            loc.kind == Link && !loc.is_covered() && loc.is_start()
        }) else {
            return None
        };
        Some(link_loc.node_handle.clone())
    }

    /// Returns a list of handles to all the nodes that we deleted
    fn replace_multiple_nodes(
        &mut self,
        range: &Range,
        new_text: S,
    ) -> Vec<DomHandle> {
        let len = new_text.len();
        let action_list = self.replace_in_text_nodes(range.clone(), new_text);

        let (to_add, to_delete, _) = action_list.grouped();
        let to_delete: Vec<DomHandle> =
            to_delete.into_iter().map(|a| a.handle).collect();

        // We only add nodes in one special case: when the selection ends at
        // a BR tag. In that case, the only nodes that might be deleted are
        // going to be before the one we add here, so their handles won't be
        // invalidated by the add we do here.
        for add_action in to_add.into_iter().rev() {
            let parent_handle = &add_action.parent_handle;
            let parent = self.state.dom.lookup_node_mut(parent_handle);
            if let DomNode::Container(parent) = parent {
                parent.insert_child(add_action.index, add_action.node);
            } else {
                panic!("Parent was not a container!");
            }
        }

        // Delete the nodes marked for deletion
        let deleted_handles = if !to_delete.is_empty() {
            self.delete_nodes(to_delete.clone())
        } else {
            Vec::new()
        };

        // If our range covered multiple text-like nodes, join together
        // the two sides of the range.
        if range.leaves().count() > 1 {
            // join_nodes will use the first location of our range, so we must
            // check whether we deleted it!
            if let Some(first_loc) = range.locations.first() {
                if !deleted_handles.contains(&first_loc.node_handle) {
                    // Calculate the position 1 code unit after the end of the
                    // range, after the in-between characters have been
                    // deleted, and the new characters have been inserted.
                    let new_pos = range.start() + len + 1;

                    // join_nodes only requires that the first location in
                    // the supplied range has a valid handle.
                    // We think it's OK to pass in a range where later
                    // locations have been deleted.
                    // TODO: can we just pass in this handle, to avoid the
                    // ambiguity here?
                    self.join_nodes(&range, new_pos);
                }
            }
        } else if let Some(first_leave) = range.leaves().next() {
            self.join_text_nodes_in_parent(
                &first_leave.node_handle.parent_handle(),
            )
        }
        deleted_handles
    }

    fn join_text_nodes_in_parent(&mut self, parent_handle: &DomHandle) {
        let child_count = if let DomNode::Container(parent) =
            self.state.dom.lookup_node(parent_handle)
        {
            parent.children().len()
        } else {
            panic!("Parent node should be a container");
        };

        if child_count > 0 {
            for i in (0..child_count - 1).rev() {
                let handle = parent_handle.child_handle(i);
                let next_handle = parent_handle.child_handle(i + 1);
                if let (DomNode::Text(cur_text), DomNode::Text(next_text)) = (
                    self.state.dom.lookup_node(&handle),
                    self.state.dom.lookup_node(&next_handle),
                ) {
                    let mut text_data = cur_text.data().to_owned();
                    let next_data = next_text.data();
                    if !next_data.is_empty() && next_data != "\u{200B}" {
                        text_data.push(next_text.data().to_owned());
                    }

                    self.state.dom.remove(&next_handle);
                    let new_text_node = DomNode::new_text(text_data);
                    self.state.dom.replace(&handle, vec![new_text_node]);
                }
            }
        }
    }

    /// Given a range to replace and some new text, modify the nodes in the
    /// range to replace the text with the supplied text.
    /// Returns a list of actions to be done to the Dom (add or remove nodes).
    /// NOTE: all nodes to be created are later in the Dom than all nodes to
    /// be deleted, so you can safely add them before performing the
    /// deletions, and the handles of the deletions will remain valid.
    fn replace_in_text_nodes(
        &mut self,
        range: Range,
        new_text: S,
    ) -> DomActionList<S> {
        let mut action_list = DomActionList::default();
        let mut first_text_node = true;

        let start = range.start();
        let end = range.end();

        for loc in range.into_iter() {
            let mut node = self.state.dom.lookup_node_mut(&loc.node_handle);
            match &mut node {
                DomNode::Container(_) => {
                    // Nothing to do for container nodes
                }
                DomNode::LineBreak(_) => {
                    match (loc.start_offset, loc.end_offset) {
                        (0, 1) => {
                            // Whole line break is selected, delete it
                            action_list.push(DomAction::remove_node(
                                loc.node_handle.clone(),
                            ));
                        }
                        (1, 1) => {
                            // Cursor is after line break, no need to delete
                        }
                        (0, 0) => {
                            if !new_text.is_empty() {
                                let node =
                                    DomNode::new_text(new_text.clone().into());
                                action_list.push(DomAction::add_node(
                                    loc.node_handle.parent_handle(),
                                    loc.node_handle.index_in_parent(),
                                    node,
                                ));
                            }
                        }
                        _ => panic!(
                            "Tried to insert text into a line break with offset != 0 or 1. \
                            Start offset: {}, end offset: {}",
                            loc.start_offset,
                            loc.end_offset,
                        ),
                    }
                    if start >= loc.position && end == loc.position + 1 {
                        // NOTE: if you add something else to `action_list` you will
                        // probably break our assumptions in the method that
                        // calls this one!
                        // We are assuming we only add nodes AFTER all the
                        // deleted nodes. (That is true in this case, because
                        // we are checking that the selection ends inside this
                        // line break.)
                        if !new_text.is_empty() {
                            action_list.push(DomAction::add_node(
                                loc.node_handle.parent_handle(),
                                loc.node_handle.index_in_parent() + 1,
                                DomNode::new_text(new_text.clone()),
                            ));
                        }
                    }
                }
                DomNode::Text(node) => {
                    let old_data = node.data();

                    // If this is not the first node, and the selections spans
                    // it, delete it.
                    if loc.start_offset == 0
                        && loc.end_offset == old_data.len()
                        && !first_text_node
                    {
                        action_list
                            .push(DomAction::remove_node(loc.node_handle));
                    } else {
                        // Otherwise, delete the selected text
                        let mut new_data =
                            old_data[..loc.start_offset].to_owned();

                        // and replace with the new content
                        if first_text_node {
                            new_data.push(new_text.deref());
                        }

                        new_data.push(&old_data[loc.end_offset..]);
                        if new_data.is_empty() {
                            action_list
                                .push(DomAction::remove_node(loc.node_handle));
                        } else {
                            node.set_data(new_data);
                        }
                    }

                    first_text_node = false;
                }
            }
        }
        action_list
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
        HashMap::from([
            (ComposerAction::Bold, ActionState::Enabled),
            (ComposerAction::Italic, ActionState::Enabled),
            (ComposerAction::StrikeThrough, ActionState::Enabled),
            (ComposerAction::Underline, ActionState::Enabled),
            (ComposerAction::InlineCode, ActionState::Enabled),
            (ComposerAction::Link, ActionState::Enabled),
            (ComposerAction::Undo, ActionState::Enabled),
            (ComposerAction::Redo, ActionState::Disabled),
            (ComposerAction::OrderedList, ActionState::Enabled),
            (ComposerAction::UnorderedList, ActionState::Enabled),
            (ComposerAction::Indent, ActionState::Disabled),
            (ComposerAction::UnIndent, ActionState::Disabled),
        ])
    }
}
