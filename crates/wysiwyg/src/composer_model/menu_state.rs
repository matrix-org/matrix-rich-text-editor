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

use crate::dom::nodes::ContainerNodeKind;
use crate::dom::Range;
use crate::menu_state::MenuStateUpdate;
use crate::{
    ComposerModel, DomHandle, DomNode, ListType, MenuState, ToolbarButton,
    UnicodeString,
};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub(crate) fn compute_menu_state(&mut self) -> MenuState {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        match range {
            Range::SameNode(range) => {
                return self.compute_active_buttons(range.node_handle);
            }
            Range::MultipleNodes(range) => {
                return self.compute_active_buttons(
                    // TODO: lookup all locations and output union instead
                    range.locations.first().unwrap().node_handle.clone(),
                );
            }
            _ => {
                return MenuState::Keep;
            }
        }
    }

    fn compute_active_buttons(&mut self, handle: DomHandle) -> MenuState {
        // TODO: handle other buttons than lists here.
        let mut active_buttons = Vec::new();
        let parent_list_item = self.state.dom.find_parent_list_item(handle);
        if let Some(list_item_handle) = parent_list_item {
            let list_handle = list_item_handle.parent_handle();
            let list_node = self.state.dom.lookup_node(list_handle);
            if let DomNode::Container(list) = list_node {
                match list.kind() {
                    ContainerNodeKind::List => {
                        let list_type =
                            ListType::try_from(list.name().clone()).unwrap();
                        match list_type {
                            ListType::Ordered => {
                                active_buttons.push(ToolbarButton::OrderedList);
                            }
                            ListType::Unordered => {
                                active_buttons
                                    .push(ToolbarButton::UnorderedList);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if active_buttons == self.active_buttons {
            return MenuState::Keep;
        } else {
            self.active_buttons = active_buttons;
            return MenuState::Update(MenuStateUpdate {
                active_buttons: self.active_buttons.clone(),
            });
        }
    }
}
