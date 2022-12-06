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

use crate::dom::nodes::dom_node::DomNodeKind::ListItem;
use crate::dom::nodes::DomNode;
use crate::dom::unicode_string::UnicodeStrExt;
use crate::dom::{DomLocation, Range};
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
                return self.create_update_replace_all();
            }
            Some(leaf) => {
                if leaf.length == 0 {
                    // Special case, there is an empty text node at the cursor position
                    self.enter_with_zero_length_selection_and_empty_text_nodes(
                        range,
                    );
                    return self.create_update_replace_all();
                } else {
                    println!("working on {:?}", leaf);
                    let current_cursor_global_location =
                        leaf.position + leaf.start_offset;
                    let handle = &leaf.node_handle;
                    let parent_list_item_handle =
                        self.state.dom.find_parent_list_item_or_self(handle);

                    match parent_list_item_handle {
                        Some(handle) => {
                            let list_item_end_offset = range
                                .locations
                                .into_iter()
                                .filter(|loc| loc.kind == ListItem)
                                .next()
                                .unwrap()
                                .end_offset;
                            self.do_enter_in_list(
                                &handle,
                                current_cursor_global_location,
                                list_item_end_offset,
                            )
                        }
                        None => {
                            self.do_enter_in_text(handle, leaf.start_offset)
                        }
                    }
                }
            }
        }
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
