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
use crate::dom::range::DomLocationPosition::{After, Before};
use crate::dom::{DomLocation, Range};
use crate::menu_state::MenuStateUpdate;
use crate::ComposerAction::{
    Indent, Link, OrderedList, Unindent, UnorderedList,
};
use crate::{
    ComposerAction, ComposerModel, DomHandle, DomNode, InlineFormatType,
    LinkAction, ListType, MenuState, UnicodeString,
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

        let reversed_actions = if let Some(first_leaf) = range.leaves().next() {
            range
                .leaves()
                // do not need locations after the cursor for next logic
                .filter(|loc| !(loc.relative_position() == After))
                .map(|loc| self.compute_reversed_actions(&loc.node_handle))
                .fold(
                    // Init with reversed_actions from the first leave.
                    self.compute_reversed_actions(&first_leaf.node_handle),
                    // And intersect with the reversed_actions of all subsequent leaves.
                    |i, set| i.intersection(&set).cloned().collect(),
                )
        } else if self.state.dom.document().children().is_empty() {
            HashSet::new()
        } else if let Some(container_loc) = range.deepest_container_node(None) {
            if container_loc.node_handle.is_root() {
                HashSet::new()
            } else {
                self.compute_reversed_actions(&container_loc.node_handle)
            }
        } else {
            HashSet::new()
        };

        reversed_actions
            .symmetric_difference(&toggled_format_actions)
            .cloned()
            .collect()
    }

    fn compute_reversed_actions(
        &self,
        handle: &DomHandle,
    ) -> HashSet<ComposerAction> {
        fn has_list_in(set: &HashSet<ComposerAction>) -> bool {
            set.contains(&OrderedList) || set.contains(&UnorderedList)
        }

        handle.with_ancestors().iter().rev().fold(
            HashSet::new(),
            |mut set, handle| {
                if let Some(action) = self.reversed_action_for_handle(handle) {
                    match action {
                        // If there is multiple list types in the hierarchy we
                        // only keep the deepest list type.
                        OrderedList | UnorderedList if has_list_in(&set) => {}
                        _ => {
                            set.insert(action);
                        }
                    }
                }
                set
            },
        )
    }

    fn reversed_action_for_handle(
        &self,
        handle: &DomHandle,
    ) -> Option<ComposerAction> {
        let node = self.state.dom.lookup_node(handle);
        if let DomNode::Container(container) = node {
            Self::reversed_action_for_container(container)
        } else {
            None
        }
    }

    fn reversed_action_for_container(
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
            ContainerNodeKind::List(list_type) => match list_type {
                ListType::Ordered => Some(ComposerAction::OrderedList),
                ListType::Unordered => Some(ComposerAction::UnorderedList),
            },
            ContainerNodeKind::CodeBlock => Some(ComposerAction::CodeBlock),
            ContainerNodeKind::Quote => Some(ComposerAction::Quote),
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
        locations: &[DomLocation],
    ) -> HashSet<ComposerAction> {
        let mut disabled_actions = HashSet::new();
        let top_most_list_locations =
            self.find_top_most_list_item_locations(locations);
        if !self.can_indent(&top_most_list_locations) {
            disabled_actions.insert(Indent);
        }
        if !self.can_unindent(&top_most_list_locations) {
            disabled_actions.insert(Unindent);
        }
        if self.get_link_action() == LinkAction::Disabled {
            disabled_actions.insert(Link);
        }
        // XOR on inline code in selection & toggled format types.
        // If selection is not a cursor, toggled format types is always
        // empty, which makes `contains_inline_code` the only condition.
        if contains_inline_code(locations)
            ^ self
                .state
                .toggled_format_types
                .contains(&InlineFormatType::InlineCode)
        {
            // Remove the rest of inline formatting options
            disabled_actions.extend(vec![
                ComposerAction::Bold,
                ComposerAction::Italic,
                ComposerAction::Underline,
                ComposerAction::StrikeThrough,
                ComposerAction::Link,
            ])
        } else if contains_code_block(locations) {
            disabled_actions.extend(vec![
                ComposerAction::InlineCode,
                ComposerAction::OrderedList,
                ComposerAction::UnorderedList,
                ComposerAction::Quote,
                ComposerAction::Link,
            ])
        }
        disabled_actions
    }
}

fn contains_inline_code(locations: &[DomLocation]) -> bool {
    locations.iter().any(|l| {
        matches!(
            l.kind,
            DomNodeKind::Formatting(InlineFormatType::InlineCode)
        )
    })
}

fn contains_code_block(locations: &[DomLocation]) -> bool {
    locations.iter().any(|l| {
        l.relative_position() != Before && l.kind == DomNodeKind::CodeBlock
    })
}
