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

use crate::dom::nodes::DomNode;
use crate::dom::unicode_string::UnicodeStrExt;
use crate::dom::{DomLocation, Range};
use crate::{ComposerModel, ComposerUpdate, UnicodeString};
use crate::dom::action_list::{DomActionList, DomAction, DomAction::Add, AddNodeAction};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn set_link(&mut self, link: S) -> ComposerUpdate<S> {
        // push_state_to_history is after this check:
        let (s, e) = self.safe_selection();
        // Can't add a link to an empty selection
        if s == e {
            return ComposerUpdate::keep();
        }
        self.push_state_to_history();

        let range = self.state.dom.find_range(s, e);
        self.set_link_range(range, link)
    }

    fn set_link_range(&mut self, range: Range, link: S) -> ComposerUpdate<S> {
        let leaves: Vec<&DomLocation> = range.leaves().collect();
        let mut actions: Vec<DomAction<S>> = vec!();
        for leaf in leaves.iter().rev() {
            let handle = &leaf.node_handle;
            // TODO: set link should be able to wrap container nodes, unlike formatting
            let node = self.state.dom.lookup_node(&handle);
            if let DomNode::Text(t) = node {
                let text = t.data();
                let before = text[..leaf.start_offset].to_owned();
                let during =
                    text[leaf.start_offset..leaf.end_offset].to_owned();
                let after = text[leaf.end_offset..].to_owned();
                let mut index = 0;
                if !before.is_empty() {
                    actions.push(Add(AddNodeAction{
                        parent_handle: handle.clone(),
                        index: index,
                        node: DomNode::new_text(before),
                    }));
                    index += 1;
                }
                actions.push(Add(AddNodeAction{
                    parent_handle: handle.clone(),
                    index: index,
                    node: DomNode::new_link(
                        link.clone(),
                        vec![DomNode::new_text(during)],
                    ),
                }));
                index += 1;
                if !after.is_empty() {
                    actions.push(Add(AddNodeAction{
                        parent_handle: handle.clone(),
                        index: index,
                        node: DomNode::new_text(after),
                    }));
                }
            } else {
                panic!("Trying to linkify a non-text node")
            }
        }
        let actions_list = DomActionList::new(actions);
        for action in actions_list.grouped().0 {
            self.state.dom.replace(&action.parent_handle, vec![action.node]);
        }
        self.create_update_replace_all()
    }
}
