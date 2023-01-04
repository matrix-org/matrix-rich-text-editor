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

use crate::{DomHandle, DomNode, UnicodeString};

/// Actions to be performed in the Dom, including all the needed info
#[derive(Clone, PartialEq, Debug)]
pub enum DomAction<S: UnicodeString> {
    Add(AddNodeAction<S>),
    Remove(RemoveNodeAction),
    Move(MoveNodeAction),
}

impl<S: UnicodeString> DomAction<S> {
    pub fn add_node(
        parent_handle: DomHandle,
        index: usize,
        node: DomNode<S>,
    ) -> Self {
        DomAction::Add(AddNodeAction {
            parent_handle,
            index,
            node,
        })
    }

    pub fn remove_node(handle: DomHandle) -> Self {
        DomAction::Remove(RemoveNodeAction { handle })
    }

    pub fn move_node(from_handle: DomHandle, to_handle: DomHandle) -> Self {
        DomAction::Move(MoveNodeAction {
            from_handle,
            to_handle,
        })
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct AddNodeAction<S: UnicodeString> {
    pub parent_handle: DomHandle,
    pub index: usize,
    pub node: DomNode<S>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RemoveNodeAction {
    pub handle: DomHandle,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MoveNodeAction {
    pub from_handle: DomHandle,
    pub to_handle: DomHandle,
}

/// Ordered list of DomActions to be performed. It contains helper methods to access different kind
/// of actions.
#[derive(Clone, PartialEq, Debug)]
pub struct DomActionList<S: UnicodeString> {
    actions: Vec<DomAction<S>>,
}

impl<S: UnicodeString> DomActionList<S> {
    #[allow(dead_code)]
    pub fn new(actions: Vec<DomAction<S>>) -> Self {
        Self { actions }
    }

    #[allow(dead_code)]
    pub fn actions(&self) -> &Vec<DomAction<S>> {
        &self.actions
    }

    #[allow(dead_code)]
    pub fn additions(&self) -> Vec<&AddNodeAction<S>> {
        self.actions
            .iter()
            .filter_map(|a| match a {
                DomAction::Add(a) => Some(a),
                _ => None,
            })
            .collect()
    }

    #[allow(dead_code)]
    pub fn removals(&self) -> Vec<&RemoveNodeAction> {
        self.actions
            .iter()
            .filter_map(|a| match a {
                DomAction::Remove(a) => Some(a),
                _ => None,
            })
            .collect()
    }

    #[allow(dead_code)]
    pub fn moves(&self) -> Vec<&MoveNodeAction> {
        self.actions
            .iter()
            .filter_map(|a| match a {
                DomAction::Move(a) => Some(a),
                _ => None,
            })
            .collect()
    }

    /// To avoid unnecessary copies, this function returns the [actions] and consumes
    /// the DomActionList.
    pub fn grouped(
        self,
    ) -> (
        Vec<AddNodeAction<S>>,
        Vec<RemoveNodeAction>,
        Vec<MoveNodeAction>,
    ) {
        let mut to_add = Vec::new();
        let mut to_remove = Vec::new();
        let mut to_move = Vec::new();

        for action in self.actions {
            match action {
                DomAction::Add(a) => to_add.push(a),
                DomAction::Remove(a) => to_remove.push(a),
                DomAction::Move(a) => to_move.push(a),
            }
        }
        (to_add, to_remove, to_move)
    }

    pub fn push(&mut self, action: DomAction<S>) {
        self.actions.push(action);
    }
    pub fn remove(&mut self, idx: usize) -> DomAction<S> {
        self.actions.remove(idx)
    }

    #[allow(dead_code)]
    pub fn replace_actions(&mut self, new_actions: Vec<DomAction<S>>) {
        self.actions = new_actions;
    }

    pub fn find_moved_parent_or_self(
        &self,
        handle: &DomHandle,
    ) -> Option<(DomHandle, DomHandle)> {
        self.actions.iter().find_map(|action| match action {
            DomAction::Move(a) => {
                if a.from_handle.is_ancestor_of(handle)
                    || a.from_handle == *handle
                {
                    Some((a.from_handle.clone(), a.to_handle.clone()))
                } else {
                    None
                }
            }
            _ => None,
        })
    }
}

impl<S: UnicodeString> Extend<DomAction<S>> for DomActionList<S> {
    fn extend<T: IntoIterator<Item = DomAction<S>>>(&mut self, iter: T) {
        self.actions.extend(iter);
    }
}

impl<S: UnicodeString> IntoIterator for DomActionList<S> {
    type Item = DomAction<S>;
    type IntoIter = std::vec::IntoIter<DomAction<S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.actions.into_iter()
    }
}

impl<S: UnicodeString> Default for DomActionList<S> {
    fn default() -> Self {
        Self {
            actions: Vec::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use widestring::Utf16String;

    type ActionList = DomActionList<Utf16String>;

    #[test]
    fn default_creates_empty_action_list() {
        let list = ActionList::default();
        assert!(list.actions.is_empty());
    }

    #[test]
    fn new_creates_action_list_with_passed_actions() {
        let actions = default_actions();
        let list = ActionList::new(actions.clone());
        assert_eq!(actions, list.actions);
    }

    #[test]
    fn new_can_create_empty_list_too() {
        let list = ActionList::new(Vec::new());
        assert!(list.actions.is_empty());
    }

    #[test]
    fn actions_fn_returns_all_actions() {
        let actions = default_actions();
        let list = ActionList::new(actions.clone());
        assert_eq!(list.actions()[0], actions[0]);
        assert_eq!(list.actions()[1], actions[1]);
        assert_eq!(list.actions()[2], actions[2]);
    }

    #[test]
    fn additions_returns_only_add_actions() {
        let actions = default_actions();
        let list = ActionList::new(actions.clone());
        assert_eq!(1, list.additions().len());
        assert_eq!(DomAction::Add(list.additions()[0].clone()), actions[0]);
    }

    #[test]
    fn removals_returns_only_remove_actions() {
        let actions = default_actions();
        let list = ActionList::new(actions.clone());
        assert_eq!(1, list.removals().len());
        assert_eq!(DomAction::Remove(list.removals()[0].clone()), actions[2]);
    }

    #[test]
    fn moves_returns_only_move_actions() {
        let actions = default_actions();
        let list = ActionList::new(actions.clone());
        assert_eq!(1, list.moves().len());
        assert_eq!(DomAction::Move(list.moves()[0].clone()), actions[1]);
    }

    #[test]
    fn push_adds_new_action_at_the_end_of_the_list() {
        let actions = default_actions();
        let mut list = ActionList::new(actions);
        let new_action = DomAction::remove_node(DomHandle::new_unset());
        list.push(new_action.clone());
        assert_eq!(list.actions[3].clone(), new_action);
    }

    #[test]
    fn replace_actions_replaces_all_actions_with_new_ones() {
        let actions = default_actions();
        let mut list = ActionList::new(actions.clone());
        list.replace_actions(Vec::new());
        assert!(list.actions.is_empty());
        list.replace_actions(actions.clone());
        assert_eq!(list.actions, actions);
    }

    #[test]
    fn find_moved_parent_or_self_looks_for_it_in_move_actions() {
        let actions = vec![DomAction::move_node(
            DomHandle::from_raw(vec![1, 0]),
            DomHandle::from_raw(vec![0, 0]),
        )];
        let list = ActionList::new(actions);
        let found_parent =
            list.find_moved_parent_or_self(&DomHandle::from_raw(vec![1, 0, 0]));
        assert_eq!(found_parent.unwrap().0, DomHandle::from_raw(vec![1, 0]));
        let found_self =
            list.find_moved_parent_or_self(&DomHandle::from_raw(vec![1, 0]));
        assert_eq!(found_self.unwrap().0, DomHandle::from_raw(vec![1, 0]));
        let not_found =
            list.find_moved_parent_or_self(&DomHandle::from_raw(vec![1]));
        assert!(not_found.is_none());
    }

    fn default_actions() -> Vec<DomAction<Utf16String>> {
        vec![
            DomAction::add_node(
                DomHandle::new_unset(),
                0,
                DomNode::new_empty_text(),
            ),
            DomAction::move_node(
                DomHandle::new_unset(),
                DomHandle::new_unset(),
            ),
            DomAction::remove_node(DomHandle::new_unset()),
        ]
    }
}
