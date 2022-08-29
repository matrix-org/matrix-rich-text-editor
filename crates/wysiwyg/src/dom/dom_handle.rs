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

#[derive(Clone, Debug, PartialEq)]
pub struct DomHandle {
    // Later, we will want to allow continuing iterating from this handle, and
    // comparing handles to see which is first in the iteration order. This
    // will allow us to walk the tree from earliest to latest of 2 handles.
    path: Vec<usize>,
}

impl DomHandle {
    pub fn from_raw(path: Vec<usize>) -> Self {
        Self { path }
    }

    pub fn parent_handle(&self) -> DomHandle {
        assert!(self.path.len() > 0);

        let mut new_path = self.path.clone();
        new_path.pop();
        DomHandle::from_raw(new_path)
    }

    pub fn child_handle(&self, child_index: usize) -> DomHandle {
        let mut new_path = self.path.clone();
        new_path.push(child_index);
        DomHandle::from_raw(new_path)
    }

    pub fn has_parent(&self) -> bool {
        self.path.len() > 0
    }

    pub fn index_in_parent(&self) -> usize {
        assert!(self.path.len() > 0);

        self.path.last().unwrap().clone()
    }

    pub fn raw(&self) -> &Vec<usize> {
        &self.path
    }

    /// Create a new UNSET handle
    ///
    /// Don't use this to lookup_node(). It will return the wrong node
    pub fn new_unset() -> Self {
        Self {
            path: vec![usize::MAX],
        }
    }

    /// Returns true if this handle has been set to a value
    pub fn is_set(&self) -> bool {
        !self.path.contains(&usize::MAX)
    }

    /// Returns true if this handle refers to a root node
    pub fn is_root(&self) -> bool {
        self.path.is_empty()
    }

    pub fn next_sibling(&self) -> Self {
        let index_in_parent = self.index_in_parent();
        let mut path = self.parent_handle().path;
        path.push(index_in_parent + 1);
        Self { path }
    }

    pub fn prev_sibling(&self) -> Self {
        let index_in_parent = self.index_in_parent();
        assert!(index_in_parent > 0);
        let mut path = self.parent_handle().path;
        path.push(index_in_parent - 1);
        Self { path }
    }
}
