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

pub mod dom_handle;
pub mod element;
pub mod html_formatter;
pub mod to_html;
pub mod range;
pub mod nodes;

use std::fmt::Display;
pub use crate::dom::nodes::container_node::ContainerNode;
pub use crate::dom::dom_handle::DomHandle;
pub use crate::dom::element::Element;
pub use crate::dom::nodes::formatting_node::FormattingNode;
pub use crate::dom::html_formatter::HtmlFormatter;
pub use crate::dom::nodes::dom_node::DomNode;
pub use crate::dom::to_html::ToHtml;
pub use crate::dom::range::{Range, SameNodeRange};
pub use crate::dom::nodes::text_node::TextNode;

fn utf8(input: &[u16]) -> String {
    String::from_utf16(input).expect("Invalid UTF-16!")
}

fn fmt_element<'a, C>(
    element: &'a impl Element<'a, C>,
    lt: C,
    gt: C,
    fwd_slash: C,
    f: &mut HtmlFormatter<C>,
) where
    C: 'static + Clone,
    DomNode<C>: ToHtml<C>,
{
    let name = element.name();
    if !name.is_empty() {
        f.write_char(&lt);
        f.write(element.name());
        f.write_char(&gt);
    }
    for child in element.children() {
        child.fmt_html(f);
    }
    if !name.is_empty() {
        f.write_char(&lt);
        f.write_char(&fwd_slash);
        f.write(element.name());
        f.write_char(&gt);
    }
}

pub fn fmt_element_u16<'a>(
    element: &'a impl Element<'a, u16>,
    f: &mut HtmlFormatter<u16>,
) {
    fmt_element(element, '<' as u16, '>' as u16, '/' as u16, f);
}

#[derive(Debug, PartialEq)]
enum FindResult {
    Found {
        node_handle: DomHandle,
        offset: usize,
    },
    NotFound {
        new_offset: usize,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dom<C> {
    document: DomNode<C>,
}

impl<C> Dom<C> {
    pub fn new(top_level_items: Vec<DomNode<C>>) -> Self {
        let mut document = ContainerNode::new(Vec::new(), top_level_items);
        document.set_handle(DomHandle::from_raw(Vec::new()));

        Self {
            document: DomNode::Container(document),
        }
    }

    fn document(&self) -> &ContainerNode<C> {
        // Would be nice if we could avoid this, but it is really convenient
        // in several places to be able to treat document as a DomNode.
        if let DomNode::Container(ret) = &self.document {
            ret
        } else {
            panic!("Document should always be a Container!")
        }
    }

    fn document_mut(&mut self) -> &mut ContainerNode<C> {
        // Would be nice if we could avoid this, but it is really convenient
        // in several places to be able to treat document as a DomNode.
        if let DomNode::Container(ret) = &mut self.document {
            ret
        } else {
            panic!("Document should always be a Container!")
        }
    }

    pub fn children(&self) -> &Vec<DomNode<C>> {
        self.document().children()
    }

    pub fn children_mut(&mut self) -> &mut Vec<DomNode<C>> {
        self.document_mut().children_mut()
    }

    pub fn append(&mut self, child: DomNode<C>) {
        self.document_mut().append(child)
    }

    pub fn replace(&mut self, node_handle: DomHandle, nodes: Vec<DomNode<C>>) {
        let parent_node = self.lookup_node_mut(node_handle.parent_handle());
        let index = node_handle.index_in_parent();
        match parent_node {
            DomNode::Text(_n) => panic!("Text nodes can't have children"),
            DomNode::Formatting(n) => n.replace_child(index, nodes),
            DomNode::Container(n) => n.replace_child(index, nodes),
        }
    }

    pub fn find_range_mut(&mut self, start: usize, end: usize) -> Range {
        if self.children().is_empty() {
            return Range::NoNode;
        }

        // Potentially silly to walk the tree twice to find both parts, but
        // care will be needed since end may be before start. Very unlikely to
        // be a performance bottleneck, so it's probably fine like this.
        let find_start = self.find_pos(self.document_handle(), start);
        let find_end = self.find_pos(self.document_handle(), end);

        // TODO: needs careful handling when on the boundary of 2 ranges:
        // we want to be greedy about when we state something is the same range
        // - maybe find_pos should return 2 nodes when we are on the boundary?
        match (find_start, find_end) {
            (
                FindResult::Found {
                    node_handle: start_handle,
                    offset: start_offset,
                },
                FindResult::Found {
                    node_handle: end_handle,
                    offset: end_offset,
                },
            ) => {
                if start_handle == end_handle {
                    Range::SameNode(SameNodeRange {
                        node_handle: start_handle,
                        start_offset,
                        end_offset,
                    })
                } else {
                    Range::TooDifficultForMe
                }
            }
            _ => Range::TooDifficultForMe,
        }
    }

    fn find_pos(&self, node_handle: DomHandle, offset: usize) -> FindResult {
        // TODO: consider whether cloning DomHandles is damaging performance,
        // and look for ways to pass around references, maybe.
        fn process_element<'a, C: 'a>(
            dom: &Dom<C>,
            element: &'a impl Element<'a, C>,
            offset: usize,
        ) -> FindResult {
            let mut off = offset;
            for child in element.children() {
                let child_handle = child.handle();
                assert!(
                    !child_handle.raw().is_empty(),
                    "Invalid child handle!"
                );
                let find_child = dom.find_pos(child_handle, off);
                //let find_child = FindResult::NotFound { new_offset: offset };
                match find_child {
                    FindResult::Found { .. } => {
                        return find_child;
                    }
                    FindResult::NotFound { new_offset } => {
                        off = new_offset;
                    }
                }
            }
            FindResult::NotFound { new_offset: off }
        }

        let node = self.lookup_node(node_handle.clone());
        match node {
            DomNode::Text(n) => {
                let len = n.data().len();
                if offset <= len {
                    FindResult::Found {
                        node_handle,
                        offset,
                    }
                } else {
                    FindResult::NotFound {
                        new_offset: offset - len,
                    }
                }
            }
            DomNode::Formatting(n) => process_element(self, n, offset),
            DomNode::Container(n) => process_element(self, n, offset),
        }
    }

    fn document_handle(&self) -> DomHandle {
        self.document.handle()
    }

    /// Find the node based on its handle.
    /// Panics if the handle is invalid
    pub fn lookup_node(&self, node_handle: DomHandle) -> &DomNode<C> {
        fn nth_child<'a, C>(
            element: &'a impl Element<'a, C>,
            idx: usize,
        ) -> &DomNode<C> {
            element.children().get(idx).expect(&format!(
                "This DomHandle wants child {} of this node, but it does \
                not have that many children.",
                idx
            ))
        }

        let mut node = &self.document;
        if !node_handle.is_valid() {
            panic!(
                "Attempting to lookup a node using an invalid DomHandle ({:?})",
                node_handle.raw()
            );
        }
        for idx in node_handle.raw() {
            node = match node {
                DomNode::Container(n) => nth_child(n, *idx),
                DomNode::Formatting(n) => nth_child(n, *idx),
                DomNode::Text(_) => panic!(
                    "Handle path looks for the child of a text node, but text \
                    nodes cannot have children."
                ),
            }
        }
        node
    }

    /// Find the node based on its handle and returns a mutable reference.
    /// Panics if the handle is invalid
    pub fn lookup_node_mut(
        &mut self,
        node_handle: DomHandle,
    ) -> &mut DomNode<C> {
        // TODO: horrible that we repeat lookup_node's logic. Can we share?
        fn nth_child<'a, C>(
            element: &'a mut impl Element<'a, C>,
            idx: usize,
        ) -> &mut DomNode<C> {
            element.children_mut().get_mut(idx).expect(&format!(
                "This DomHandle wants child {} of this node, but it does \
                not have that many children.",
                idx
            ))
        }

        let mut node = &mut self.document;
        for idx in node_handle.raw() {
            node = match node {
                DomNode::Container(n) => nth_child(n, *idx),
                DomNode::Formatting(n) => nth_child(n, *idx),
                DomNode::Text(_) => panic!(
                    "Handle path looks for the child of a text node, but text \
                    nodes cannot have children."
                ),
            }
        }
        node
    }
}

impl<C> ToHtml<C> for Dom<C>
    where
        C: Clone,
        ContainerNode<C>: ToHtml<C>,
{
    fn fmt_html(&self, f: &mut HtmlFormatter<C>) {
        self.document().fmt_html(f)
    }
}

impl Display for Dom<u16> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&utf8(&self.to_html()))?;
        Ok(())
    }
}



impl<'a, C> Element<'a, C> for FormattingNode<C> {
    fn name(&'a self) -> &'a Vec<C> {
        &self.name
    }

    fn children(&'a self) -> &'a Vec<DomNode<C>> {
        &self.children
    }

    fn children_mut(&'a mut self) -> &'a mut Vec<DomNode<C>> {
        &mut self.children
    }
}

impl ToHtml<u16> for FormattingNode<u16> {
    fn fmt_html(&self, f: &mut HtmlFormatter<u16>) {
        fmt_element_u16(self, f)
    }
}

/* TODO
#[derive(Clone, Debug, PartialEq)]
struct ItemNode {}

impl Display for ItemNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
*/

#[cfg(test)]
mod test {
    use crate::dom::nodes::dom_node::DomNode;
    use super::*;

    fn utf16(input: &str) -> Vec<u16> {
        input.encode_utf16().collect()
    }

    fn clone_children<'a, C>(
        children: impl IntoIterator<Item = &'a DomNode<C>>,
    ) -> Vec<DomNode<C>>
        where
            C: 'static + Clone,
    {
        children.into_iter().cloned().collect()
    }

    fn dom<'a, C>(children: impl IntoIterator<Item = &'a DomNode<C>>) -> Dom<C>
        where
            C: 'static + Clone,
    {
        Dom::new(clone_children(children))
    }

    fn b<'a>(
        children: impl IntoIterator<Item = &'a DomNode<u16>>,
    ) -> DomNode<u16> {
        DomNode::Formatting(FormattingNode::new(
            utf16("b"),
            clone_children(children),
        ))
    }

    fn i<'a>(
        children: impl IntoIterator<Item = &'a DomNode<u16>>,
    ) -> DomNode<u16> {
        DomNode::Formatting(FormattingNode::new(
            utf16("i"),
            clone_children(children),
        ))
    }

    fn tx(data: &str) -> DomNode<u16> {
        DomNode::Text(TextNode::from(utf16(data)))
    }

    /// If this node is an element, return its children - otherwise panic
    fn kids<C>(node: &DomNode<C>) -> &Vec<DomNode<C>> {
        match node {
            DomNode::Container(n) => n.children(),
            DomNode::Formatting(n) => n.children(),
            DomNode::Text(_) => {
                panic!("We expected an Element, but found Text")
            }
        }
    }

    // Creation and handles

    #[test]
    fn can_create_a_dom_and_add_nodes() {
        // Create a simple DOM
        let dom = Dom::new(vec![
            DomNode::Text(TextNode::from("a".to_html())),
            DomNode::Formatting(FormattingNode::new(
                "b".to_html(),
                vec![DomNode::Text(TextNode::from("b".to_html()))],
            )),
        ]);

        // The DOM was created successfully
        assert_eq!(dom.to_string(), "a<b>b</b>");
    }

    #[test]
    fn can_find_toplevel_nodes_via_handles() {
        // Create a simple DOM
        let dom = Dom::new(vec![
            DomNode::Text(TextNode::from("a".to_html())),
            DomNode::Formatting(FormattingNode::new(
                "b".to_html(),
                vec![DomNode::Text(TextNode::from("b".to_html()))],
            )),
        ]);

        let child0 = &dom.children()[0];
        let child1 = &dom.children()[1];

        // The handles point to the right nodes
        assert_eq!(dom.lookup_node(child0.handle()), child0);
        assert_eq!(dom.lookup_node(child1.handle()), child1);
    }

    #[test]
    fn can_find_deep_nodes_via_handles() {
        let dom = dom(&[
            tx("foo"),
            b(&[tx("BOLD"), b(&[tx("uberbold")])]),
            tx("bar"),
        ]);

        // Given a DOM with a nested node
        let nested_node = &kids(&kids(&dom.children()[1])[1])[0];

        // When we ask for its handle
        let handle = nested_node.handle();

        // Then we can look it up and find the same node
        assert_eq!(dom.lookup_node(handle), nested_node);
    }

    #[test]
    fn can_replace_toplevel_node_with_multiple_nodes() {
        let mut dom = dom(&[tx("foo"), tx("bar")]);

        let node = &dom.children()[0];
        let inserted_nodes = vec![tx("ab"), b(&[tx("cd")]), tx("ef")];

        dom.replace(node.handle(), inserted_nodes);

        // Node is replaced by new insertion
        assert_eq!(dom.to_string(), "ab<b>cd</b>efbar");
        // Subsequent node handle is properly updated
        let bar_node = &dom.children()[3];
        assert_eq!(bar_node.handle().index_in_parent(), 3);
    }

    #[test]
    fn can_replace_deep_node_with_multiple_nodes() {
        let mut dom = dom(&[b(&[tx("foo")])]);

        let node = &kids(&dom.children()[0])[0];
        let inserted_nodes = vec![tx("f"), i(&[tx("o")]), tx("o")];

        dom.replace(node.handle(), inserted_nodes);

        // Node is replaced by new insertion
        assert_eq!(dom.to_string(), "<b>f<i>o</i>o</b>");
    }

    // Serialisation

    #[test]
    fn empty_dom_serialises_to_empty_string() {
        assert_eq!(dom(&[]).to_string(), "");
    }

    #[test]
    fn plain_text_serialises_to_just_the_text() {
        assert_eq!(dom(&[tx("foo")]).to_string(), "foo");
    }

    #[test]
    fn mixed_text_and_tags_serialises() {
        assert_eq!(
            dom(&[tx("foo"), b(&[tx("BOLD")]), tx("bar")]).to_string(),
            "foo<b>BOLD</b>bar"
        );
    }

    #[test]
    fn nested_tags_serialise() {
        assert_eq!(
            dom(&[
                tx("foo"),
                b(&[tx("BO"), i(&[tx("LD")])]),
                i(&[tx("it")]),
                tx("bar")
            ])
                .to_string(),
            "foo<b>BO<i>LD</i></b><i>it</i>bar"
        );
    }

    #[test]
    fn empty_tag_serialises() {
        assert_eq!(dom(&[b(&[]),]).to_string(), "<b></b>");
    }

    // Finding nodes

    #[test]
    fn finding_a_node_within_an_empty_dom_returns_not_found() {
        let d: Dom<u16> = dom(&[]);
        assert_eq!(
            d.find_pos(d.document_handle(), 0),
            FindResult::NotFound { new_offset: 0 }
        );
    }

    #[test]
    fn finding_a_node_within_a_single_text_node_is_found() {
        let d: Dom<u16> = dom(&[tx("foo")]);
        assert_eq!(
            d.find_pos(d.document_handle(), 1),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![0]),
                offset: 1
            }
        );
    }

    #[test]
    fn finding_a_node_within_flat_text_nodes_is_found() {
        let d: Dom<u16> = dom(&[tx("foo"), tx("bar")]);
        assert_eq!(
            d.find_pos(d.document_handle(), 0),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![0]),
                offset: 0
            }
        );
        assert_eq!(
            d.find_pos(d.document_handle(), 1),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![0]),
                offset: 1
            }
        );
        assert_eq!(
            d.find_pos(d.document_handle(), 2),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![0]),
                offset: 2
            }
        );
        // TODO: selections at boundaries need work
        /*
        assert_eq!(
            d.find_pos(d.document_handle(), 3),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![1]),
                offset: 0
            }
        );
        assert_eq!(
            d.find_pos(d.document_handle(), 4),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![1]),
                offset: 1
            }
        );
        assert_eq!(
            d.find_pos(d.document_handle(), 5),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![1]),
                offset: 2
            }
        );
        */
        assert_eq!(
            d.find_pos(d.document_handle(), 6),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![1]),
                offset: 3
            }
        );
    }

    // TODO: comprehensive test like above for non-flat nodes

    #[test]
    fn finding_a_range_within_an_empty_dom_returns_no_node() {
        let mut d: Dom<u16> = dom(&[]);
        let range = d.find_range_mut(0, 0);
        assert_eq!(range, Range::NoNode);
    }

    #[test]
    fn finding_a_range_within_the_single_text_node_works() {
        let mut d = dom(&[tx("foo bar baz")]);
        let range = d.find_range_mut(4, 7);

        if let Range::SameNode(range) = range {
            assert_eq!(range.start_offset, 4);
            assert_eq!(range.end_offset, 7);

            if let DomNode::Text(t) = d.lookup_node(range.node_handle.clone()) {
                assert_eq!(t.data(), "foo bar baz".to_html());
            } else {
                panic!("Should have been a text node!")
            }

            assert_eq!(range.node_handle.raw(), &vec![0]);
        } else {
            panic!("Should have been a SameNodeRange: {:?}", range)
        }
    }

    #[test]
    fn finding_a_range_that_includes_the_end_works_simple_case() {
        let mut d = dom(&[tx("foo bar baz")]);
        let range = d.find_range_mut(4, 11);

        if let Range::SameNode(range) = range {
            assert_eq!(range.start_offset, 4);
            assert_eq!(range.end_offset, 11);

            if let DomNode::Text(t) = d.lookup_node(range.node_handle.clone()) {
                assert_eq!(t.data(), "foo bar baz".to_html());
            } else {
                panic!("Should have been a text node!")
            }

            assert_eq!(range.node_handle.raw(), &vec![0]);
        } else {
            panic!("Should have been a SameNodeRange: {:?}", range)
        }
    }

    #[test]
    fn finding_a_range_within_some_nested_node_works() {
        let mut d = dom(&[tx("foo "), b(&[tx("bar")]), tx(" baz")]);
        let range = d.find_range_mut(5, 6);

        if let Range::SameNode(range) = range {
            assert_eq!(range.start_offset, 1);
            assert_eq!(range.end_offset, 2);

            if let DomNode::Text(t) = d.lookup_node(range.node_handle.clone()) {
                assert_eq!(t.data(), "bar".to_html());
            } else {
                panic!("Should have been a text node!")
            }

            assert_eq!(range.node_handle.raw(), &vec![1, 0]);
        } else {
            panic!("Should have been a SameNodeRange: {:?}", range)
        }
    }

    /*#[test]
    fn finding_range_within_complex_tags_doesnt_work_yet() {
        // TODO: we can't do this yet
        let d = dom(&[tx("foo "), b(&[tx("bar")]), tx(" baz")]);
        let range = d.find_range(4, 7);
        assert_eq!(range, Range::TooDifficultForMe);
    }*/

    // TODO: copy tests from platforms/web/example/test.js
    // TODO: improve tests when we have HTML parsing
}
