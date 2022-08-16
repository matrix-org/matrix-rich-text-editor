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

use crate::dom::dom_handle::DomHandle;
use crate::dom::element::Element;
use crate::dom::nodes::dom_node::DomNode;

#[derive(Clone, Debug, PartialEq)]
pub struct FormattingNode<C> {
    pub name: Vec<C>,
    pub children: Vec<DomNode<C>>,
    handle: DomHandle,
}

impl<C> FormattingNode<C> {
    /// Create a new FormattingNode
    ///
    /// NOTE: Its handle() will be invalid until you call set_handle() or
    /// append() it to another node.
    pub fn new(name: Vec<C>, children: Vec<DomNode<C>>) -> Self {
        Self {
            name,
            children,
            handle: DomHandle::new_invalid(),
        }
    }

    pub fn handle(&self) -> DomHandle {
        self.handle.clone()
    }

    pub fn set_handle(&mut self, handle: DomHandle) {
        // TODO: copied into 2 places - move into Element?
        self.handle = handle;
        for (i, child) in self.children.iter_mut().enumerate() {
            child.set_handle(self.handle.child_handle(i))
        }
    }

    pub fn append(&mut self, mut child: DomNode<C>) {
        assert!(self.handle.is_valid());
        // TODO: copied into 2 places - move into Element?

        let child_index = self.children.len();
        let child_handle = self.handle.child_handle(child_index);
        child.set_handle(child_handle);
        self.children.push(child);
    }

    pub fn replace_child(&mut self, index: usize, nodes: Vec<DomNode<C>>) {
        assert!(self.handle.is_valid());
        assert!(index < self.children().len());
        // TODO: copied into 2 places - move into Element?

        self.children.remove(index);
        let mut current_index = index;
        for mut node in nodes {
            let child_handle = self.handle.child_handle(current_index);
            node.set_handle(child_handle);
            self.children.insert(current_index, node);
            current_index += 1;
        }

        for child_index in current_index..self.children.len() {
            let new_handle = self.handle.child_handle(child_index);
            self.children[child_index].set_handle(new_handle);
        }
    }
}
