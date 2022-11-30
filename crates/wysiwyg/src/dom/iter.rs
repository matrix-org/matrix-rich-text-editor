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

use crate::{DomNode, UnicodeString};

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
}

impl<S> DomNode<S>
where
    S: UnicodeString,
{
    /// Return an iterator over all nodes of the subtree starting from this
    /// node (including self), in depth-first order
    pub fn iter(&self) -> DomIterator<S> {
        DomIterator::over(self)
    }

    /// Return an iterator over all text nodes of the subtree starting from
    /// this node (including self), in depth-first order
    pub fn iter_text(&self) -> impl Iterator<Item = &TextNode<S>> {
        self.iter().filter_map(DomNode::as_text)
    }

    /// Return an iterator over all container nodes of this DOM, in depth-first
    /// order
    pub fn iter_containers(&self) -> impl Iterator<Item = &ContainerNode<S>> {
        self.iter().filter_map(DomNode::as_container)
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

impl<'a, S> DomIterator<'a, S>
where
    S: UnicodeString,
{
    fn over(dom_node: &'a DomNode<S>) -> Self {
        Self {
            started: false,
            ancestors: vec![NodeAndChildIndex {
                node: &dom_node,
                child_index: 0,
            }],
        }
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
    use crate::DomNode;

    const EXAMPLE_HTML: &str = "\
        <ul>\
            <li>b<strong>c</strong></li>\
            <li>foo</li>\
        </ul>\
        <i>d</i>e|<br />\
        <b>x</b>";

    #[test]
    fn can_walk_all_nodes() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let text_nodes: Vec<String> = dom.iter().map(node_txt).collect();

        assert_eq!(
            text_nodes,
            vec![
                "", "ul", "li", "'b'", "strong", "'c'", "li", "'foo'", "i",
                "'d'", "'e'", "br", "b", "'x'"
            ]
        );
    }

    #[test]
    fn can_walk_all_nodes_of_a_leading_subtree() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let first_child = dom.children().first().unwrap();
        let text_nodes: Vec<String> =
            first_child.iter().map(node_txt).collect();

        assert_eq!(
            text_nodes,
            vec!["ul", "li", "'b'", "strong", "'c'", "li", "'foo'"]
        )
    }

    #[test]
    fn can_walk_all_nodes_of_a_middle_subtree() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let second_child = &dom.children()[1];
        let text_nodes: Vec<String> =
            second_child.iter().map(node_txt).collect();

        assert_eq!(text_nodes, vec!["i", "'d'"])
    }

    #[test]
    fn can_walk_all_nodes_of_a_trailing_subtree() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let last_child = dom.children().last().unwrap();
        let text_nodes: Vec<String> = last_child.iter().map(node_txt).collect();

        assert_eq!(text_nodes, vec!["b", "'x'"])
    }

    #[test]
    fn can_walk_all_nodes_of_a_deep_subtree() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        if let DomNode::Container(list) = dom.children().first().unwrap() {
            let deep_child = list.children().first().unwrap();
            let text_nodes: Vec<String> =
                deep_child.iter().map(node_txt).collect();

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
            .iter_text()
            .map(|text| text.data().to_string())
            .collect();

        assert_eq!(text_nodes, vec!["b", "c", "foo"])
    }

    #[test]
    fn can_walk_all_text_nodes_of_a_middle_subtree() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let second_child = &dom.children()[1];
        let text_nodes: Vec<String> = second_child
            .iter_text()
            .map(|text| text.data().to_string())
            .collect();

        assert_eq!(text_nodes, vec!["d"])
    }

    #[test]
    fn can_walk_all_text_nodes_of_a_trailing_subtree() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let last_child = dom.children().last().unwrap();
        let text_nodes: Vec<String> = last_child
            .iter_text()
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
                .iter_text()
                .map(|text| text.data().to_string())
                .collect();

            assert_eq!(text_nodes, vec!["b", "c"])
        } else {
            panic!("First child should have been the list")
        }
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
            vec!["", "ul", "li", "strong", "li", "i", "b"]
        );
    }

    fn node_txt(node: &DomNode<Utf16String>) -> String {
        match node {
            DomNode::Container(c) => c.name().to_string(),
            DomNode::Text(t) => format!("'{}'", t.data()),
            DomNode::LineBreak(_) => String::from("br"),
        }
    }
}
