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

use strum::IntoEnumIterator;

use crate::action_state::ActionState;
use crate::dom::nodes::dom_node::DomNodeKind;
use crate::dom::nodes::{ContainerNode, ContainerNodeKind};
use crate::dom::{DomLocation, Range};
use crate::menu_state::MenuStateUpdate;
use crate::ComposerAction::{Indent, UnIndent};
use crate::{
    ComposerAction, ComposerModel, DomHandle, DomNode, InlineFormatType,
    ListType, MenuState, UnicodeString,
};
use std::collections::{HashMap, HashSet};

pub(crate) enum MenuStateComputeType {
    AlwaysUpdate,
    KeepIfUnchanged,
}

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub(crate) fn compute_menu_state(
        &mut self,
        compute_type: MenuStateComputeType,
    ) -> MenuState {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);

        let action_states = self.compute_action_states(&range);

        if action_states == self.action_states
            && matches!(compute_type, MenuStateComputeType::KeepIfUnchanged)
        {
            MenuState::Keep
        } else {
            self.action_states = action_states.clone();
            MenuState::Update(MenuStateUpdate { action_states })
        }
    }

    fn compute_action_states(
        &self,
        range: &Range,
    ) -> HashMap<ComposerAction, ActionState> {
        let mut action_states = HashMap::new();

        let reversed = self.compute_reversed_actions_from_range(range);
        let disabled = self.compute_disabled_actions();

        for action in ComposerAction::iter() {
            let state = if disabled.contains(&action) {
                ActionState::Disabled
            } else if reversed.contains(&action) {
                ActionState::Reversed
            } else {
                ActionState::Enabled
            };
            action_states.insert(action, state);
        }

        action_states
    }

    fn compute_reversed_actions_from_range(
        &self,
        range: &Range,
    ) -> HashSet<ComposerAction> {
        let toggled_format_actions = self
            .state
            .toggled_format_types
            .iter()
            .map(|format| format.action())
            .collect();

        if let Some(intersection) = range.leaves().next().map(|l| {
            range
                .leaves()
                // do not need locations after the cursor for next logic
                .filter(|loc| loc.position < range.end())
                .fold(
                    // Init with reversed_actions from the first leave.
                    self.compute_reversed_actions(&l.node_handle),
                    // And intersect with the reversed_actions of all subsequent leaves.
                    |set, leave| {
                        set.intersection(
                            &self.compute_reversed_actions(&leave.node_handle),
                        )
                        .cloned()
                        .collect()
                    },
                )
                .symmetric_difference(&toggled_format_actions)
                .cloned()
                .collect()
        }) {
            intersection
        } else if self.state.dom.document().children().is_empty() {
            HashSet::from_iter(toggled_format_actions.into_iter())
        } else {
            HashSet::new()
        }
    }

    fn compute_reversed_actions(
        &self,
        handle: &DomHandle,
    ) -> HashSet<ComposerAction> {
        let mut reversed_actions = HashSet::new();
        let node = self.state.dom.lookup_node(handle);
        if let DomNode::Container(container) = node {
            let active_button = Self::active_button_for_container(container);
            if let Some(button) = active_button {
                reversed_actions.insert(button);
            }
        }

        if handle.has_parent() {
            reversed_actions = reversed_actions
                .union(&self.compute_reversed_actions(&handle.parent_handle()))
                .into_iter()
                .cloned()
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
                    ListType::try_from(container.name().to_owned()).unwrap();
                match list_type {
                    ListType::Ordered => Some(ComposerAction::OrderedList),
                    ListType::Unordered => Some(ComposerAction::UnorderedList),
                }
            }
            ContainerNodeKind::CodeBlock => Some(ComposerAction::CodeBlock),
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

        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        disabled_actions.extend(
            self.compute_disabled_actions_for_locations(&range.locations),
        );
        disabled_actions
    }

    fn compute_disabled_actions_for_locations(
        &self,
        locations: &Vec<DomLocation>,
    ) -> HashSet<ComposerAction> {
        let mut disabled_actions = HashSet::new();
        if !self.can_indent(locations) {
            disabled_actions.insert(Indent);
        }
        if !self.can_unindent(locations) {
            disabled_actions.insert(UnIndent);
        }
        let contains_inline_code_node = locations
            .iter()
            .find(|l| {
                matches!(
                    l.kind,
                    DomNodeKind::Formatting(InlineFormatType::InlineCode)
                )
            })
            .is_some();
        if contains_inline_code_node {
            // Remove the rest of inline formatting options
            disabled_actions.extend(vec![
                ComposerAction::Bold,
                ComposerAction::Italic,
                ComposerAction::Underline,
                ComposerAction::StrikeThrough,
                ComposerAction::Link,
            ])
        }
        disabled_actions
    }
}
