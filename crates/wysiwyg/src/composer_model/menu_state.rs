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

use crate::dom::nodes::{ContainerNode, ContainerNodeKind};
use crate::dom::{DomLocation, Range};
use crate::menu_state::MenuStateUpdate;
use crate::{
    ComposerModel, DomHandle, DomNode, InlineFormatType, ListType, MenuState,
    ToolbarButton, UnicodeString,
};
use std::collections::HashSet;

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub(crate) fn compute_menu_state(&mut self) -> MenuState {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        let active_buttons: HashSet<ToolbarButton>;
        match range {
            Range::SameNode(range) => {
                active_buttons = self.compute_active_buttons(range.node_handle);
            }
            Range::MultipleNodes(range) => {
                active_buttons = self
                    .compute_active_buttons_from_locations(&range.locations);
            }
            _ => {
                active_buttons = HashSet::new();
            }
        }
        let disabled_buttons = self.compute_disabled_buttons();

        if active_buttons == self.active_buttons
            && disabled_buttons == self.disabled_buttons
        {
            return MenuState::Keep;
        } else {
            self.active_buttons = active_buttons;
            self.disabled_buttons = disabled_buttons;
            return MenuState::Update(MenuStateUpdate {
                active_buttons: self.active_buttons.clone(),
                disabled_buttons: self.disabled_buttons.clone(),
            });
        }
    }

    fn compute_active_buttons_from_locations(
        &mut self,
        locations: &Vec<DomLocation>,
    ) -> HashSet<ToolbarButton> {
        let mut text_locations: Vec<&DomLocation> = locations
            .iter()
            .filter(|l| {
                let node = self.state.dom.lookup_node(&l.node_handle);
                node.is_text_node()
            })
            .collect();
        let first_location = text_locations.remove(0);
        let mut active_buttons =
            self.compute_active_buttons(first_location.node_handle.clone());
        for location in text_locations {
            let buttons =
                self.compute_active_buttons(location.node_handle.clone());
            let intersection: HashSet<_> = active_buttons
                .intersection(&buttons)
                .into_iter()
                .map(|b| b.clone())
                .collect();
            active_buttons = intersection;
        }

        active_buttons
    }

    fn compute_active_buttons(
        &self,
        handle: DomHandle,
    ) -> HashSet<ToolbarButton> {
        let mut active_buttons = HashSet::new();
        let node = self.state.dom.lookup_node(&handle);
        if let DomNode::Container(container) = node {
            let active_button = Self::active_button_for_container(container);
            if let Some(button) = active_button {
                active_buttons.insert(button);
            }
        }

        if handle.has_parent() {
            active_buttons = active_buttons
                .union(&self.compute_active_buttons(handle.parent_handle()))
                .into_iter()
                .map(|b| b.clone())
                .collect();
        }

        active_buttons
    }

    fn active_button_for_container(
        container: &ContainerNode<S>,
    ) -> Option<ToolbarButton> {
        match container.kind() {
            ContainerNodeKind::Formatting(format) => match format {
                InlineFormatType::Bold => Some(ToolbarButton::Bold),
                InlineFormatType::Italic => Some(ToolbarButton::Italic),
                InlineFormatType::StrikeThrough => {
                    Some(ToolbarButton::StrikeThrough)
                }
                InlineFormatType::Underline => Some(ToolbarButton::Underline),
                InlineFormatType::InlineCode => Some(ToolbarButton::InlineCode),
            },
            ContainerNodeKind::Link(_) => Some(ToolbarButton::Link),
            ContainerNodeKind::List => {
                let list_type =
                    ListType::try_from(container.name().clone()).unwrap();
                match list_type {
                    ListType::Ordered => Some(ToolbarButton::OrderedList),
                    ListType::Unordered => Some(ToolbarButton::UnorderedList),
                }
            }
            _ => None,
        }
    }

    fn compute_disabled_buttons(&self) -> HashSet<ToolbarButton> {
        let mut disabled_buttons = HashSet::new();
        if self.previous_states.is_empty() {
            disabled_buttons.insert(ToolbarButton::Undo);
        }
        if self.next_states.is_empty() {
            disabled_buttons.insert(ToolbarButton::Redo);
        }

        disabled_buttons
    }
}
