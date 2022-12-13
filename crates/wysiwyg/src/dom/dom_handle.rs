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

#[derive(Clone, Debug, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct DomHandle {
    // The location of a node in the tree, or None if we don't know yet
    path: Option<Vec<usize>>,
}

impl DomHandle {
    /// Create a new handle for the root/document node.
    pub fn root() -> Self {
        Self {
            path: Some(Vec::new()),
        }
    }

    /// Create a new handle with the provided path.
    /// So long as the path provided points to a valid node, this handle
    /// can be used (it is set).
    pub fn from_raw(path: Vec<usize>) -> Self {
        Self { path: Some(path) }
    }

    /// Create a new UNSET handle
    /// Don't use this handle for anything until it has been set.
    /// Most methods will panic!
    pub fn new_unset() -> Self {
        Self { path: None }
    }

    /// Returns true if this handle has been set to a value
    pub fn is_set(&self) -> bool {
        self.path.is_some()
    }

    /// Returns true if this handle refers to a root node
    /// Panics if this handle is unset.
    pub fn is_root(&self) -> bool {
        self.raw().is_empty()
    }

    /// Returns the depth of the handle.
    /// Panics if this handle is unset.  
    pub fn depth(&self) -> usize {
        self.path.as_ref().expect("Handle is unset!").len()
    }

    /// Return the handle of this node's parent, or None if this is the
    /// root node.
    /// Panics if this handle is unset
    /// Panics if we have no parent (i.e. this handle is the root)
    pub fn parent_handle(&self) -> DomHandle {
        let path = self.raw();
        if path.is_empty() {
            panic!("Root handle has no parent!");
        } else {
            let mut new_path = path.clone();
            new_path.pop();
            DomHandle::from_raw(new_path)
        }
    }

    /// Return a new handle for one of our children, with the supplied index.
    /// Panics if this handle is unset
    pub fn child_handle(&self, child_index: usize) -> DomHandle {
        let mut new_path = self.raw().clone();
        new_path.push(child_index);
        DomHandle::from_raw(new_path)
    }

    /// Returns a DomHandle with the 'sub-path' up to the passed 'depth'.
    /// i.e.: a DomHandle with path `[0, 1, 2, 3]` with `at_depth(2)` will return a DomHandle
    /// with path `[0, 1, 2]`.
    pub fn sub_handle_up_to(&self, depth: usize) -> DomHandle {
        assert!(&self.path.is_some());
        let path = self.path.clone().unwrap();
        let (new_path, _) = path.split_at(depth);
        DomHandle::from_raw(new_path.to_vec())
    }

    /// Return true if this handle has a parent i.e. it is not the root. If
    /// this returns true, it is safe to call index_in_parent() and
    /// parent_handle().
    /// Panics if this handle is unset
    pub fn has_parent(&self) -> bool {
        !self.raw().is_empty()
    }

    /// Return this handle's position within its parent.
    /// Panics if this handle is unset
    /// Panics if we have no parent (i.e. this handle is the root)
    pub fn index_in_parent(&self) -> usize {
        *self.raw().last().expect("Root handle has no parent!")
    }

    /// Return the underlying path used to represent this handle.
    /// Panics if this handle is unset
    pub fn raw(&self) -> &Vec<usize> {
        self.path.as_ref().expect("Handle is unset!")
    }

    /// Consume self and return the underlying path.
    /// Panics if this handle is unset
    pub fn into_raw(self) -> Vec<usize> {
        self.path.expect("Handle is unset!")
    }

    /// Returns a handle to an invalid location if this is the last child
    pub fn next_sibling(&self) -> Self {
        let index_in_parent = self.index_in_parent();
        let mut path = self.parent_handle().into_raw();
        path.push(index_in_parent + 1);
        Self::from_raw(path)
    }

    /// Panics if this is the first child
    pub fn prev_sibling(&self) -> Self {
        let index_in_parent = self.index_in_parent();
        assert!(index_in_parent > 0);
        let mut path = self.parent_handle().into_raw();
        path.push(index_in_parent - 1);
        Self::from_raw(path)
    }

    /// Returns true if the passed handle is an ancestor of the current one, but false if it is
    /// either unrelated to it or it's the same handle.
    pub fn is_ancestor_of(&self, other: &DomHandle) -> bool {
        let own_path = self.raw();
        let other_path = other.raw();
        other_path.starts_with(own_path.as_slice()) && other_path != own_path
    }

    /// Replaces the sub-path shared with [old] handle with the same sub-path in [new].
    pub fn replace_ancestor(&mut self, old: DomHandle, new: DomHandle) {
        assert!(old.is_ancestor_of(self) || old == *self);
        assert!(self.is_set());
        let mut new_path = self.path.as_ref().unwrap().clone();
        new_path.splice(0..old.raw().len(), new.into_raw());
        self.path = Some(new_path.clone());
    }
}

#[cfg(test)]
mod test {
    use crate::DomHandle;

    #[test]
    fn creating_root_handle() {
        let root = DomHandle::root();
        assert!(root.is_root());
        assert!(!root.is_ancestor_of(&DomHandle::root()));
        assert!(root.is_ancestor_of(&DomHandle::from_raw(vec![0, 1, 2])));
    }

    #[test]
    #[should_panic]
    fn lookup_parent_of_root_panics() {
        DomHandle::root().parent_handle();
    }

    #[test]
    #[should_panic]
    fn computing_sibling_of_root_panics() {
        DomHandle::root().next_sibling();
    }

    #[test]
    fn computing_handle_depth() {
        assert_eq!(DomHandle::root().depth(), 0);
        assert_eq!(DomHandle::from_raw(vec![0]).depth(), 1);
        assert_eq!(DomHandle::from_raw(vec![0, 1, 2]).depth(), 3);
    }

    #[test]
    #[should_panic]
    fn computing_unset_handle_depth_panics() {
        DomHandle::new_unset().depth();
    }

    #[test]
    fn computing_parent_handle() {
        let handle = DomHandle::from_raw(vec![0, 1, 2]);
        assert!(handle.has_parent());
        assert_eq!(handle.index_in_parent(), 2);
        assert_eq!(handle.parent_handle().index_in_parent(), 1);
        assert_eq!(handle.parent_handle().raw(), &vec![0, 1]);
        assert_eq!(
            &handle.parent_handle().parent_handle().parent_handle(),
            &DomHandle::root(),
        );
    }

    #[test]
    fn sub_handle_up_to_depth() {
        let handle = DomHandle::from_raw(vec![0, 1, 2, 1, 2]);
        assert_eq!(handle.sub_handle_up_to(3).raw(), &vec![0, 1, 2]);
        assert_eq!(handle.sub_handle_up_to(handle.depth()), handle)
    }

    #[test]
    #[should_panic]
    fn sub_handle_up_to_unreachable_depth_panics() {
        let handle = DomHandle::from_raw(vec![0, 1, 2, 1, 2]);
        handle.sub_handle_up_to(10);
    }

    #[test]
    fn computing_siblings_handle() {
        let handle = DomHandle::from_raw(vec![0, 2, 1]);
        assert_eq!(handle.prev_sibling().raw(), &vec![0, 2, 0]);
        assert_eq!(handle.next_sibling().raw(), &vec![0, 2, 2]);
    }

    #[test]
    #[should_panic]
    fn computing_prev_sibling_of_first_child_panics() {
        DomHandle::from_raw(vec![0, 1, 0]).prev_sibling();
    }

    #[test]
    fn replacing_handle_ancestor() {
        let mut handle = DomHandle::from_raw(vec![0, 1, 2, 4, 5]);
        let parent = DomHandle::from_raw(vec![0, 1, 2]);
        let new_parent = DomHandle::from_raw(vec![7, 8]);
        handle.replace_ancestor(parent, new_parent);
        assert_eq!(handle.raw(), &vec![7, 8, 4, 5],)
    }

    #[test]
    fn replacing_handle_ancestor_using_self() {
        let mut handle = DomHandle::from_raw(vec![0, 1, 2, 4, 5]);
        let new_parent = DomHandle::from_raw(vec![7, 8]);
        handle.replace_ancestor(handle.clone(), new_parent.clone());
        assert_eq!(handle.raw(), new_parent.raw());
    }

    #[test]
    #[should_panic]
    fn replacing_handle_ancestor_panics_if_not_using_ancestor() {
        let mut handle = DomHandle::from_raw(vec![0, 1, 2, 4, 5]);
        let parent = DomHandle::from_raw(vec![0, 1, 4]);
        let new_parent = DomHandle::from_raw(vec![7, 8]);
        handle.replace_ancestor(parent, new_parent);
    }
}
