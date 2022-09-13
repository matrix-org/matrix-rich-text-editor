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
    ComposerAction, ComposerModel, DomHandle, DomNode, InlineFormatType,
    ListType, MenuState, UnicodeString,
};
use std::collections::HashSet;

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub(crate) fn compute_menu_state(&mut self) -> MenuState {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        let reversed_actions: HashSet<ComposerAction>;
        match range {
            Range::SameNode(range) => {
                reversed_actions =
                    self.compute_reversed_actions(range.node_handle);
            }
            Range::MultipleNodes(range) => {
                reversed_actions = self
                    .compute_reversed_actions_from_locations(&range.locations);
            }
            _ => {
                reversed_actions = HashSet::new();
            }
        }
        let disabled_actions = self.compute_disabled_actions();

        if reversed_actions == self.reversed_actions
            && disabled_actions == self.disabled_actions
        {
            return MenuState::Keep;
        } else {
            self.reversed_actions = reversed_actions;
            self.disabled_actions = disabled_actions;
            return MenuState::Update(MenuStateUpdate {
                reversed_actions: self.reversed_actions.clone(),
                disabled_actions: self.disabled_actions.clone(),
            });
        }
    }

    fn compute_reversed_actions_from_locations(
        &mut self,
        locations: &Vec<DomLocation>,
    ) -> HashSet<ComposerAction> {
        let mut text_locations: Vec<&DomLocation> = locations
            .iter()
            .filter(|l| {
                let node = self.state.dom.lookup_node(&l.node_handle);
                node.is_text_node()
            })
            .collect();
        let first_location = text_locations.remove(0);
        let mut reversed_actions =
            self.compute_reversed_actions(first_location.node_handle.clone());
        for location in text_locations {
            let buttons =
                self.compute_reversed_actions(location.node_handle.clone());
            let intersection: HashSet<_> = reversed_actions
                .intersection(&buttons)
                .into_iter()
                .map(|b| b.clone())
                .collect();
            reversed_actions = intersection;
        }

        reversed_actions
    }

    fn compute_reversed_actions(
        &self,
        handle: DomHandle,
    ) -> HashSet<ComposerAction> {
        let mut reversed_actions = HashSet::new();
        let node = self.state.dom.lookup_node(&handle);
        if let DomNode::Container(container) = node {
            let active_button = Self::active_button_for_container(container);
            if let Some(button) = active_button {
                reversed_actions.insert(button);
            }
        }

        if handle.has_parent() {
            reversed_actions = reversed_actions
                .union(&self.compute_reversed_actions(handle.parent_handle()))
                .into_iter()
                .map(|b| b.clone())
                .collect();
        }

        reversed_actions
    }

    fn active_button_for_container(
        container: &ContainerNode<S>,
    ) -> Option<ComposerAction> {
        match container.kind() {
            ContainerNodeKind::Formatting(format) => match format {
                InlineFormatType::Bold => Some(ComposerAction::Bold),
                InlineFormatType::Italic => Some(ComposerAction::Italic),
                InlineFormatType::StrikeThrough => {
                    Some(ComposerAction::StrikeThrough)
                }
                InlineFormatType::Underline => Some(ComposerAction::Underline),
                InlineFormatType::InlineCode => {
                    Some(ComposerAction::InlineCode)
                }
            },
            ContainerNodeKind::Link(_) => Some(ComposerAction::Link),
            ContainerNodeKind::List => {
                let list_type =
                    ListType::try_from(container.name().clone()).unwrap();
                match list_type {
                    ListType::Ordered => Some(ComposerAction::OrderedList),
                    ListType::Unordered => Some(ComposerAction::UnorderedList),
                }
            }
            _ => None,
        }
    }

    fn compute_disabled_actions(&self) -> HashSet<ComposerAction> {
        let mut disabled_actions = HashSet::new();
        if self.previous_states.is_empty() {
            disabled_actions.insert(ComposerAction::Undo);
        }
        if self.next_states.is_empty() {
            disabled_actions.insert(ComposerAction::Redo);
        }

        disabled_actions
    }
}
