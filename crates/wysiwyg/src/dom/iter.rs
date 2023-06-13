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

//! Methods on Dom that modify its contents and are guaranteed to conform to
//! our invariants:
//! * No empty text nodes
//! * No adjacent text nodes
//! * No empty containers
//! * List items must be inside lists

use crate::{DomHandle, DomNode};
use std::collections::HashSet;
use unicode_string::UnicodeString;

use super::{
    nodes::{ContainerNode, TextNode},
    Dom,
};

impl<S> Dom<S>
where
    S: UnicodeString,
{
    /// Return an iterator over all nodes of this DOM, in depth-first order
    pub fn iter(&self) -> DomIterator<S> {
        DomIterator::over(self.document_node())
    }

    /// Return an iterator over all text nodes of this DOM, in depth-first
    /// order
    pub fn iter_text(&self) -> impl Iterator<Item = &TextNode<S>> {
        self.iter().filter_map(DomNode::as_text)
    }

    /// Return an iterator over all container nodes of this DOM, in depth-first
    /// order
    pub fn iter_containers(&self) -> impl Iterator<Item = &ContainerNode<S>> {
        self.iter().filter_map(DomNode::as_container)
    }

    /// Return an iterator over all nodes of the DOM from the passed node,
    /// depth-first order (including self).
    pub fn iter_from<'a>(&'a self, node: &'a DomNode<S>) -> DomNodeIterator<S> {
        DomNodeIterator::over(self, node)
    }

    /// Return an iterator over all nodes of the DOM from the passed DomHandle,
    /// depth-first order (including self).
    pub fn iter_from_handle(&self, handle: &DomHandle) -> DomNodeIterator<S> {
        DomNodeIterator::over(self, self.lookup_node(handle))
    }

    /// Return an iterator over all handles of the DOM from the passed handle,
    /// depth-first order (including self).
    pub fn handle_iter_from<'a>(
        &'a self,
        handle: &'a DomHandle,
    ) -> DomHandleIterator<S> {
        DomHandleIterator::over(self, handle)
    }

    /// Return an iterator over all text nodes of the DOM from the passed node,
    /// depth-first order (including self).
    pub fn iter_text_from<'a>(
        &'a self,
        node: &'a DomNode<S>,
    ) -> impl Iterator<Item = &TextNode<S>> {
        self.iter_from(node).filter_map(DomNode::as_text)
    }

    /// Return the previous node in the DOM, if exists, in depth-first order.
    pub fn prev_node(&mut self, handle: &DomHandle) -> Option<&DomNode<S>> {
        let mut iter = self.iter_from_handle(handle);
        iter.next_back(); // Current node
        iter.next_back()
    }

    /// Return the previous leaf node in the DOM, if exists, in depth-first order.
    pub fn prev_leaf(&mut self, handle: &DomHandle) -> Option<&DomNode<S>> {
        let mut iter = self.iter_from_handle(handle);
        iter.next_back(); // Current node
        while let Some(prev) = iter.next_back() {
            if prev.is_leaf() {
                return Some(prev);
            }
        }
        None
    }

    /// Return the handle of the previous node in the DOM, if exists, in depth-first order.
    pub fn prev_handle(&mut self, handle: &DomHandle) -> Option<DomHandle> {
        let mut iter = self.iter_from_handle(handle);
        iter.next_back(); // Current node
        let Some(prev) = iter.next_back() else {
            return None;
        };
        Some(prev.handle())
    }

    /// Return the next node in the DOM, if exists, in depth-first order.
    pub fn next_node(&mut self, handle: &DomHandle) -> Option<&DomNode<S>> {
        let mut iter = self.iter_from_handle(handle);
        iter.next(); // Current node
        iter.next()
    }

    /// Return the next leaf node in the DOM, if exists, in depth-first order.
    pub fn next_leaf(&mut self, handle: &DomHandle) -> Option<&DomNode<S>> {
        let mut iter = self.iter_from_handle(handle);
        iter.next(); // Current node
        iter.find(|&next| next.is_leaf())
    }

    /// Return the handle of the next node in the DOM, if exists, in depth-first order.
    pub fn next_handle(&mut self, handle: &DomHandle) -> Option<DomHandle> {
        let mut iter = self.iter_from_handle(handle);
        iter.next(); // Current node
        let Some(next) = iter.next() else {
            return None;
        };
        Some(next.handle())
    }
}

impl<S> DomNode<S>
where
    S: UnicodeString,
{
    /// Return an iterator over all nodes of the subtree starting from this
    /// node (including self), in depth-first order
    pub fn iter_subtree(&self) -> DomIterator<S> {
        DomIterator::over(self)
    }

    /// Return an iterator over all text nodes of the subtree starting from
    /// this node (including self), in depth-first order
    pub fn iter_text_in_subtree(&self) -> impl Iterator<Item = &TextNode<S>> {
        self.iter_subtree().filter_map(DomNode::as_text)
    }

    /// Return an iterator over all container nodes of this DOM, in depth-first
    /// order
    pub fn iter_containers(&self) -> impl Iterator<Item = &ContainerNode<S>> {
        self.iter_subtree().filter_map(DomNode::as_container)
    }
}

/// A DomNode and the index of its child that we are currently processing.
struct NodeAndChildIndex<'a, S>
where
    S: UnicodeString,
{
    node: &'a DomNode<S>,
    child_index: usize,
}

pub struct DomIterator<'a, S>
where
    S: UnicodeString,
{
    started: bool,
    ancestors: Vec<NodeAndChildIndex<'a, S>>,
}

pub struct DomNodeIterator<'a, S>
where
    S: UnicodeString,
{
    dom: &'a Dom<S>,
    handle_iterator: DomHandleIterator<'a, S>,
}

pub struct DomHandleIterator<'a, S>
where
    S: UnicodeString,
{
    started: bool,
    dom: &'a Dom<S>,
    current: Option<DomHandle>,
    visited: HashSet<DomHandle>,
}

impl<'a, S> DomNodeIterator<'a, S>
where
    S: UnicodeString,
{
    fn over(dom: &'a Dom<S>, dom_node: &'a DomNode<S>) -> Self {
        Self {
            dom,
            handle_iterator: DomHandleIterator::over(dom, &dom_node.handle()),
        }
    }
}

impl<'a, S> Iterator for DomNodeIterator<'a, S>
where
    S: UnicodeString,
{
    type Item = &'a DomNode<S>;

    fn next(&mut self) -> Option<Self::Item> {
        self.handle_iterator
            .next()
            .map(|h| self.dom.lookup_node(&h))
    }
}

impl<'a, S> DoubleEndedIterator for DomNodeIterator<'a, S>
where
    S: UnicodeString,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.handle_iterator
            .next_back()
            .map(|h| self.dom.lookup_node(&h))
    }
}

impl<'a, S> DomHandleIterator<'a, S>
where
    S: UnicodeString,
{
    fn over(dom: &'a Dom<S>, handle: &DomHandle) -> Self {
        Self {
            started: false,
            dom,
            current: Some(handle.clone()),
            visited: HashSet::new(),
        }
    }

    fn next_sibling_or_parent(&self, handle: &DomHandle) -> Option<DomHandle> {
        if !handle.has_parent() {
            return None;
        }
        let parent = self.dom.lookup_node(&handle.parent_handle());
        let DomNode::Container(c) = parent else {
            panic!("Parent node must be a container");
        };
        let idx = handle.index_in_parent() + 1;
        if idx < c.children().len() {
            c.children().get(idx).map(|n| n.handle())
        } else if parent.handle().has_parent() {
            self.next_sibling_or_parent(&parent.handle())
        } else {
            None
        }
    }

    fn prev_sibling_or_parent(
        &mut self,
        handle: &DomHandle,
    ) -> Option<DomHandle> {
        if !handle.has_parent() {
            return None;
        }
        let parent = self.dom.lookup_node(&handle.parent_handle());
        let DomNode::Container(c) = parent else {
            panic!("Parent node must be a container");
        };
        let idx = handle.index_in_parent();
        if idx > 0 {
            c.children().get(idx - 1).map(|n| n.handle())
        } else if parent.handle().has_parent() {
            if !self.visited.contains(&parent.handle()) {
                self.visited.insert(parent.handle());
                Some(parent.handle())
            } else {
                self.prev_sibling_or_parent(&parent.handle())
            }
        } else if !self.visited.contains(&parent.handle()) {
            self.visited.insert(parent.handle());
            Some(parent.handle())
        } else {
            None
        }
    }
}

impl<'a, S> DomIterator<'a, S>
where
    S: UnicodeString,
{
    fn over(dom_node: &'a DomNode<S>) -> Self {
        Self {
            started: false,
            ancestors: vec![NodeAndChildIndex {
                node: dom_node,
                child_index: 0,
            }],
        }
    }
}

impl<'a, S> Iterator for DomHandleIterator<'a, S>
where
    S: UnicodeString,
{
    type Item = DomHandle;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(current) = self.current.clone() else {
            return None;
        };
        if self.started {
            if let DomNode::Container(c) = self.dom.lookup_node(&current) {
                let first_child_handle = current.child_handle(0);
                if !c.children().is_empty()
                    && !self.visited.contains(&first_child_handle)
                {
                    self.current = Some(first_child_handle);
                } else {
                    self.current = self.next_sibling_or_parent(&current);
                }
            } else if current.has_parent() {
                self.current = self.next_sibling_or_parent(&current);
            }
        } else {
            self.started = true;
        }
        self.visited.insert(current);
        self.current.clone()
    }
}

impl<'a, S> DoubleEndedIterator for DomHandleIterator<'a, S>
where
    S: UnicodeString,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let Some(current) = self.current.clone() else {
            return None;
        };
        if self.started {
            if let DomNode::Container(c) = self.dom.lookup_node(&current) {
                // Don't go deeper if the current container node has been visited
                if !self.visited.contains(&current)
                    && !c.children().is_empty()
                    && !self
                        .visited
                        .contains(&c.children().last().unwrap().handle())
                {
                    self.current =
                        Some(current.child_handle(c.children().len() - 1));
                } else if current.has_parent() {
                    self.current = self.prev_sibling_or_parent(&current);
                } else {
                    self.current = None;
                }
            } else if current.has_parent() {
                self.current = self.prev_sibling_or_parent(&current);
            }
        } else {
            self.started = true;
        }
        self.visited.insert(current);
        self.current.clone()
    }
}

impl<'a, S> Iterator for DomIterator<'a, S>
where
    S: UnicodeString,
{
    type Item = &'a DomNode<S>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.started {
            let parent = self.ancestors.iter_mut().last();
            if let Some(NodeAndChildIndex {
                node: DomNode::Container(c),
                child_index: idx,
            }) = parent
            {
                let siblings = c.children();
                if *idx < siblings.len() {
                    let myself = &siblings[*idx];
                    *idx += 1;
                    if let DomNode::Container(_) = myself {
                        self.ancestors.push(NodeAndChildIndex {
                            node: myself,
                            child_index: 0,
                        });
                    }
                    Some(myself)
                } else {
                    self.ancestors.pop();
                    self.next()
                }
            } else {
                None
            }
        } else {
            self.started = true;
            Some(self.ancestors[0].node)
        }
    }
}

#[cfg(test)]
mod test {
    use widestring::Utf16String;

    use crate::tests::testutils_composer_model::cm;
    use crate::{DomHandle, DomNode};

    const EXAMPLE_HTML: &str = "\
        <ul>\
            <li>b<strong>c</strong></li>\
            <li>foo</li>\
        </ul>\
        <p>\
            <i>d</i>e|<br />\
            <b>x</b>\
        </p>";

    #[test]
    fn can_walk_all_nodes() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let text_nodes: Vec<String> = dom.iter().map(node_txt).collect();

        assert_eq!(
            text_nodes,
            vec![
                "", "ul", "li", "'b'", "strong", "'c'", "li", "'foo'", "p",
                "i", "'d'", "'e'", "br", "b", "'x'"
            ]
        );
    }

    #[test]
    fn can_walk_all_nodes_of_a_leading_subtree() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let first_child = dom.children().first().unwrap();
        let text_nodes: Vec<String> =
            first_child.iter_subtree().map(node_txt).collect();

        assert_eq!(
            text_nodes,
            vec!["ul", "li", "'b'", "strong", "'c'", "li", "'foo'"]
        )
    }

    #[test]
    fn can_walk_all_nodes_of_a_middle_subtree() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let sub_tree = dom.lookup_node(&DomHandle::from_raw(vec![1, 0]));
        let text_nodes: Vec<String> =
            sub_tree.iter_subtree().map(node_txt).collect();

        assert_eq!(text_nodes, vec!["i", "'d'"])
    }

    #[test]
    fn can_walk_all_nodes_of_a_trailing_subtree() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let sub_tree = dom.lookup_node(&DomHandle::from_raw(vec![1, 3]));
        let text_nodes: Vec<String> =
            sub_tree.iter_subtree().map(node_txt).collect();

        assert_eq!(text_nodes, vec!["b", "'x'"])
    }

    #[test]
    fn can_walk_all_nodes_of_a_deep_subtree() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        if let DomNode::Container(list) = dom.children().first().unwrap() {
            let deep_child = list.children().first().unwrap();
            let text_nodes: Vec<String> =
                deep_child.iter_subtree().map(node_txt).collect();

            assert_eq!(text_nodes, vec!["li", "'b'", "strong", "'c'"])
        } else {
            panic!("First child should have been the list")
        }
    }

    #[test]
    fn can_walk_all_text_nodes() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let text_nodes: Vec<String> = dom
            .iter_text()
            .map(|text| text.data().to_string())
            .collect();

        assert_eq!(text_nodes, vec!["b", "c", "foo", "d", "e", "x"]);
    }

    #[test]
    fn can_walk_all_text_nodes_of_a_leading_subtree() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let first_child = dom.children().first().unwrap();
        let text_nodes: Vec<String> = first_child
            .iter_text_in_subtree()
            .map(|text| text.data().to_string())
            .collect();

        assert_eq!(text_nodes, vec!["b", "c", "foo"])
    }

    #[test]
    fn can_walk_all_text_nodes_of_a_middle_subtree() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let sub_tree = &dom.lookup_node(&DomHandle::from_raw(vec![1, 0]));
        let text_nodes: Vec<String> = sub_tree
            .iter_text_in_subtree()
            .map(|text| text.data().to_string())
            .collect();

        assert_eq!(text_nodes, vec!["d"])
    }

    #[test]
    fn can_walk_all_text_nodes_of_a_trailing_subtree() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let sub_tree = &dom.lookup_node(&DomHandle::from_raw(vec![1, 3]));
        let text_nodes: Vec<String> = sub_tree
            .iter_text_in_subtree()
            .map(|text| text.data().to_string())
            .collect();

        assert_eq!(text_nodes, vec!["x"])
    }

    #[test]
    fn can_walk_all_text_nodes_of_a_deep_subtree() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        if let DomNode::Container(list) = dom.children().first().unwrap() {
            let deep_child = list.children().first().unwrap();
            let text_nodes: Vec<String> = deep_child
                .iter_text_in_subtree()
                .map(|text| text.data().to_string())
                .collect();

            assert_eq!(text_nodes, vec!["b", "c"])
        } else {
            panic!("First child should have been the list")
        }
    }

    #[test]
    fn can_walk_all_nodes_of_the_tree_from_a_node() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let first_child = dom.children().first().unwrap();
        let text_nodes: Vec<String> =
            dom.iter_from(first_child).map(node_txt).collect();

        assert_eq!(
            text_nodes,
            vec![
                "ul", "li", "'b'", "strong", "'c'", "li", "'foo'", "p", "i",
                "'d'", "'e'", "br", "b", "'x'"
            ]
        )
    }

    #[test]
    fn can_walk_all_nodes_of_the_tree_from_a_leaf_node() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let handle = DomHandle::from_raw(vec![0, 0, 1, 0]);
        let text_nodes: Vec<String> =
            dom.iter_from_handle(&handle).map(node_txt).collect();

        assert_eq!(
            text_nodes,
            vec![
                "'c'", "li", "'foo'", "p", "i", "'d'", "'e'", "br", "b", "'x'"
            ]
        )
    }

    #[test]
    fn can_walk_all_nodes_of_the_tree_from_root() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let text_nodes: Vec<String> =
            dom.iter_from(dom.document_node()).map(node_txt).collect();

        assert_eq!(
            text_nodes,
            vec![
                "", "ul", "li", "'b'", "strong", "'c'", "li", "'foo'", "p",
                "i", "'d'", "'e'", "br", "b", "'x'"
            ]
        )
    }

    #[test]
    fn can_walk_all_nodes_of_the_tree_from_last_node_in_dom() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let handle = dom.last_node_handle();
        let text_nodes: Vec<String> =
            dom.iter_from_handle(&handle).map(node_txt).collect();

        assert_eq!(text_nodes, vec!["'x'"])
    }

    #[test]
    fn can_walk_all_nodes_of_the_tree_from_last_node_in_dom_reversed() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let handle = dom.last_node_handle();
        let text_nodes: Vec<String> =
            dom.iter_from_handle(&handle).rev().map(node_txt).collect();

        assert_eq!(
            text_nodes,
            vec![
                "'x'", "b", "br", "'e'", "i", "'d'", "p", "ul", "li", "'foo'",
                "li", "strong", "'c'", "'b'", "",
            ]
        )
    }

    #[test]
    fn can_walk_all_nodes_of_the_tree_from_root_reversed() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let text_nodes: Vec<String> = dom
            .iter_from(dom.document_node())
            .rev()
            .map(node_txt)
            .collect();

        assert_eq!(text_nodes, vec![""])
    }

    #[test]
    fn can_walk_all_nodes_of_the_tree_from_first_child_reversed() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let text_nodes: Vec<String> = dom
            .iter_from(dom.document().children().first().unwrap())
            .rev()
            .map(node_txt)
            .collect();

        assert_eq!(text_nodes, vec!["ul", ""])
    }

    #[test]
    fn can_walk_all_nodes_of_the_tree_from_a_deep_leaf_node_reversed() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let handle = DomHandle::from_raw(vec![0, 0, 1, 0]);
        let text_nodes: Vec<String> =
            dom.iter_from_handle(&handle).rev().map(node_txt).collect();

        assert_eq!(text_nodes, vec!["'c'", "strong", "'b'", "li", "ul", ""])
    }

    #[test]
    fn can_walk_all_nodes_of_the_tree_from_a_middle_leaf_node_reversed() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let handle = DomHandle::from_raw(vec![0, 1, 0]);
        let text_nodes: Vec<String> =
            dom.iter_from_handle(&handle).rev().map(node_txt).collect();

        assert_eq!(
            text_nodes,
            vec!["'foo'", "li", "li", "strong", "'c'", "'b'", "ul", ""]
        )
    }

    #[test]
    fn can_walk_all_nodes_of_the_tree_from_a_node_reversed() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        // Start from the last <b> tag.
        let handle = DomHandle::from_raw(vec![1, 3]);
        let last_child = dom.lookup_node(&handle);
        let text_nodes: Vec<String> =
            dom.iter_from(last_child).rev().map(node_txt).collect();

        assert_eq!(
            text_nodes,
            vec![
                "b", "br", "'e'", "i", "'d'", "p", "ul", "li", "'foo'", "li",
                "strong", "'c'", "'b'", ""
            ]
        )
    }

    #[test]
    fn can_walk_all_container_nodes() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let container_nodes: Vec<String> = dom
            .iter_containers()
            .map(|c| c.name().to_string())
            .collect();

        assert_eq!(
            container_nodes,
            vec!["", "ul", "li", "strong", "li", "p", "i", "b"]
        );
    }

    fn node_txt(node: &DomNode<Utf16String>) -> String {
        match node {
            DomNode::Container(c) => c.name().to_string(),
            DomNode::Text(t) => format!("'{}'", t.data()),
            DomNode::LineBreak(_) => String::from("br"),
            DomNode::Mention(_) => String::from("mention"),
        }
    }
}
