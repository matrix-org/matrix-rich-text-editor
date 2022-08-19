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

pub mod dom_creation_error;
pub mod dom_handle;
pub mod find_result;
pub mod html_formatter;
pub mod nodes;
pub mod parser;
pub mod range;
pub mod to_html;

pub use dom_creation_error::DomCreationError;
pub use dom_handle::DomHandle;
pub use find_result::FindResult;
pub use html_formatter::HtmlFormatter;
pub use range::Range;
pub use range::SameNodeRange;
pub use to_html::ToHtml;

use self::nodes::{ContainerNode, ContainerNodeKind};
use self::nodes::{DomNode, TextNode};

use std::fmt::Display;

fn utf8(input: &[u16]) -> String {
    String::from_utf16(input).expect("Invalid UTF-16!")
}

fn fmt_node<C>(
    node: &ContainerNode<C>,
    lt: C,
    gt: C,
    equal: C,
    quote: C,
    space: C,
    fwd_slash: C,
    f: &mut HtmlFormatter<C>,
) where
    C: 'static + Clone,
    DomNode<C>: ToHtml<C>,
{
    let name = node.name();
    if !name.is_empty() {
        f.write_char(&lt);
        f.write(node.name());
        if let Some(attrs) = node.attributes() {
            for attr in attrs {
                f.write_char(&space);
                let (name, value) = attr;
                f.write(name);
                f.write_char(&equal);
                f.write_char(&quote);
                f.write(value);
                f.write_char(&quote);
            }
        }
        f.write_char(&gt);
    }

    for child in node.children() {
        child.fmt_html(f);
    }
    if !name.is_empty() {
        f.write_char(&lt);
        f.write_char(&fwd_slash);
        f.write(node.name());
        f.write_char(&gt);
    }
}

pub fn fmt_node_u16(node: &ContainerNode<u16>, f: &mut HtmlFormatter<u16>) {
    fmt_node(
        node, '<' as u16, '>' as u16, '=' as u16, '"' as u16, ' ' as u16,
        '/' as u16, f,
    );
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dom<C>
where
    C: Clone,
{
    document: DomNode<C>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum RangeLocation {
    Start,
    End,
}

impl<C> Dom<C>
where
    C: Clone,
{
    pub fn new(top_level_items: Vec<DomNode<C>>) -> Self {
        let mut document = ContainerNode::new(
            Vec::new(),
            ContainerNodeKind::Generic,
            None,
            top_level_items,
        );
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
            DomNode::Container(n) => n.replace_child(index, nodes),
        }
    }

    pub fn find_range(&self, start: usize, end: usize) -> Range {
        if self.children().is_empty() {
            return Range::NoNode;
        }

        // TODO: We walk the whole tree twice (by calling find_pos twice) -
        // maybe we can do better than that?  (But very unlikely to be a
        // performance problem.)

        // TODO: more tests that directly exercise this beginning and end stuff
        let (find_start, find_end) = match start.cmp(&end) {
            std::cmp::Ordering::Equal => {
                // When there is no range, only a cursor, we use "end" style,
                // staying within a tag if we are near the end
                let pos = self.find_pos(
                    self.document_handle(),
                    end,
                    RangeLocation::End,
                );
                (pos.clone(), pos)
            }
            std::cmp::Ordering::Less => {
                // Start and end are in expected order - use normal start
                // and end style for find them.
                (
                    self.find_pos(
                        self.document_handle(),
                        start,
                        RangeLocation::Start,
                    ),
                    self.find_pos(
                        self.document_handle(),
                        end,
                        RangeLocation::End,
                    ),
                )
            }
            std::cmp::Ordering::Greater => {
                // Start and end are in opposite order - use opposite start
                // and end style for find them.
                (
                    self.find_pos(
                        self.document_handle(),
                        start,
                        RangeLocation::End,
                    ),
                    self.find_pos(
                        self.document_handle(),
                        end,
                        RangeLocation::Start,
                    ),
                )
            }
        };

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

    /// Find a particular character position in the DOM
    ///
    /// location controls whether we are looking for the start or the end
    /// of a range. When we are on the border of a tag, if we are looking for
    /// the start, we return the character at the beginning of the next tag,
    /// whereas if we are looking for the end of a range, we return the
    /// position after the last character of the previous tag.
    ///
    /// When searching for an individual character (rather than a range), you
    /// should ask for RangeLocation::End.
    fn find_pos(
        &self,
        node_handle: DomHandle,
        offset: usize,
        location: RangeLocation,
    ) -> FindResult {
        // TODO: move this function into its own file
        // TODO: consider whether cloning DomHandles is damaging performance,
        // and look for ways to pass around references, maybe.
        fn process_container_node<C: Clone>(
            dom: &Dom<C>,
            node: &ContainerNode<C>,
            offset: usize,
            location: RangeLocation,
        ) -> FindResult {
            let mut off = offset;
            for child in node.children() {
                let child_handle = child.handle();
                assert!(!child_handle.is_root(), "Incorrect child handle!");
                let find_child = dom.find_pos(child_handle, off, location);
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
                if (location == RangeLocation::Start && offset < len)
                    || (location == RangeLocation::End && offset <= len)
                {
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
            DomNode::Container(n) => {
                process_container_node(self, n, offset, location)
            }
        }
    }

    fn document_handle(&self) -> DomHandle {
        self.document.handle()
    }

    /// Find the node based on its handle.
    /// Panics if the handle is unset or invalid
    pub fn lookup_node(&self, node_handle: DomHandle) -> &DomNode<C> {
        fn nth_child<C: Clone>(
            element: &ContainerNode<C>,
            idx: usize,
        ) -> &DomNode<C> {
            element.children().get(idx).expect(
                "Handle is invalid: it refers to a child index which is too \
                large for the number of children in this node.",
            )
        }

        let mut node = &self.document;
        if !node_handle.is_set() {
            panic!(
                "Attempting to lookup a node using an unset DomHandle ({:?})",
                node_handle.raw()
            );
        }
        for idx in node_handle.raw() {
            node = match node {
                DomNode::Container(n) => nth_child(n, *idx),
                DomNode::Text(_) => panic!(
                    "Handle is invalid: refers to the child of a text node, \
                    but text nodes cannot have children."
                ),
            }
        }
        node
    }

    /// Find the node based on its handle and returns a mutable reference.
    /// Panics if the handle is invalid or unset
    pub fn lookup_node_mut(
        &mut self,
        node_handle: DomHandle,
    ) -> &mut DomNode<C> {
        // TODO: horrible that we repeat lookup_node's logic. Can we share?
        fn nth_child<C: Clone>(
            element: &mut ContainerNode<C>,
            idx: usize,
        ) -> &mut DomNode<C> {
            element.children_mut().get_mut(idx).expect(
                "Handle is invalid: it refers to a child index which is too \
                large for the number of children in this node.",
            )
        }

        let mut node = &mut self.document;
        if !node_handle.is_set() {
            panic!(
                "Attempting to lookup a node using an unset DomHandle ({:?})",
                node_handle.raw()
            );
        }
        for idx in node_handle.raw() {
            node = match node {
                DomNode::Container(n) => nth_child(n, *idx),
                DomNode::Text(_) => panic!(
                    "Handle is invalid: refers to the child of a text node, \
                    but text nodes cannot have children."
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
    use super::*;
    use crate::dom::nodes::dom_node::DomNode;

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

    fn a<'a>(
        children: impl IntoIterator<Item = &'a DomNode<u16>>,
    ) -> DomNode<u16> {
        DomNode::new_link(utf16("https://element.io"), clone_children(children))
    }

    fn b<'a>(
        children: impl IntoIterator<Item = &'a DomNode<u16>>,
    ) -> DomNode<u16> {
        DomNode::new_formatting(utf16("b"), clone_children(children))
    }

    fn i<'a>(
        children: impl IntoIterator<Item = &'a DomNode<u16>>,
    ) -> DomNode<u16> {
        DomNode::new_formatting(utf16("i"), clone_children(children))
    }

    fn i_c<'a>(
        children: impl IntoIterator<Item = &'a DomNode<u16>>,
    ) -> DomNode<u16> {
        DomNode::new_formatting(utf16("code"), clone_children(children))
    }

    fn tx(data: &str) -> DomNode<u16> {
        DomNode::Text(TextNode::from(utf16(data)))
    }

    /// If this node is an element, return its children - otherwise panic
    fn kids<C: Clone>(node: &DomNode<C>) -> &Vec<DomNode<C>> {
        match node {
            DomNode::Container(n) => n.children(),
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
            DomNode::new_formatting(
                "b".to_html(),
                vec![DomNode::Text(TextNode::from("b".to_html()))],
            ),
        ]);

        // The DOM was created successfully
        assert_eq!(dom.to_string(), "a<b>b</b>");
    }

    #[test]
    fn can_find_toplevel_nodes_via_handles() {
        // Create a simple DOM
        let dom = Dom::new(vec![
            DomNode::Text(TextNode::from("a".to_html())),
            DomNode::new_formatting(
                "b".to_html(),
                vec![DomNode::Text(TextNode::from("b".to_html()))],
            ),
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

        let node_handle = dom.children()[0].handle();
        let inserted_nodes = vec![tx("ab"), b(&[tx("cd")]), tx("ef")];

        dom.replace(node_handle, inserted_nodes);

        // Node is replaced by new insertion
        assert_eq!(dom.to_string(), "ab<b>cd</b>efbar");
        // Subsequent node handle is properly updated
        let bar_node = &dom.children()[3];
        assert_eq!(bar_node.handle().index_in_parent(), 3);
    }

    #[test]
    fn can_replace_deep_node_with_multiple_nodes() {
        let mut dom = dom(&[b(&[tx("foo")])]);

        let node_handle = kids(&dom.children()[0])[0].handle();
        let inserted_nodes = vec![tx("f"), i(&[tx("o")]), tx("o")];

        dom.replace(node_handle, inserted_nodes);

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

    // TODO: more tests for start and end of ranges

    #[test]
    fn finding_a_node_within_an_empty_dom_returns_not_found() {
        let d: Dom<u16> = dom(&[]);
        assert_eq!(
            d.find_pos(d.document_handle(), 0, RangeLocation::Start),
            FindResult::NotFound { new_offset: 0 }
        );
    }

    #[test]
    fn finding_a_node_within_a_single_text_node_is_found() {
        let d: Dom<u16> = dom(&[tx("foo")]);
        assert_eq!(
            d.find_pos(d.document_handle(), 1, RangeLocation::Start),
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
            d.find_pos(d.document_handle(), 0, RangeLocation::Start),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![0]),
                offset: 0
            }
        );
        assert_eq!(
            d.find_pos(d.document_handle(), 1, RangeLocation::Start),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![0]),
                offset: 1
            }
        );
        assert_eq!(
            d.find_pos(d.document_handle(), 2, RangeLocation::Start),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![0]),
                offset: 2
            }
        );
        assert_eq!(
            d.find_pos(d.document_handle(), 3, RangeLocation::Start),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![1]),
                offset: 0
            }
        );
        assert_eq!(
            d.find_pos(d.document_handle(), 3, RangeLocation::End),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![0]),
                offset: 3
            }
        );
        // TODO: break up this test and name parts!
        assert_eq!(
            d.find_pos(d.document_handle(), 4, RangeLocation::Start),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![1]),
                offset: 1
            }
        );
        assert_eq!(
            d.find_pos(d.document_handle(), 4, RangeLocation::End),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![1]),
                offset: 1
            }
        );
        assert_eq!(
            d.find_pos(d.document_handle(), 5, RangeLocation::Start),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![1]),
                offset: 2
            }
        );
        assert_eq!(
            d.find_pos(d.document_handle(), 5, RangeLocation::End),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![1]),
                offset: 2
            }
        );
        assert_eq!(
            d.find_pos(d.document_handle(), 6, RangeLocation::End),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![1]),
                offset: 3
            }
        );
    }

    // TODO: comprehensive test like above for non-flat nodes

    #[test]
    fn finding_a_range_within_an_empty_dom_returns_no_node() {
        let d: Dom<u16> = dom(&[]);
        let range = d.find_range(0, 0);
        assert_eq!(range, Range::NoNode);
    }

    #[test]
    fn finding_a_range_within_the_single_text_node_works() {
        let d = dom(&[tx("foo bar baz")]);
        let range = d.find_range(4, 7);

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
        let d = dom(&[tx("foo bar baz")]);
        let range = d.find_range(4, 11);

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
        let d = dom(&[tx("foo "), b(&[tx("bar")]), tx(" baz")]);
        let range = d.find_range(5, 6);

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

    #[test]
    fn hyperlink_formatting_simple() {
        let d = dom(&[a(&[tx("foo")])]);
        let mut formatter = HtmlFormatter::new();
        fmt_node_u16(d.document(), &mut formatter);
        assert_eq!(
            "<a href=\"https://element.io\">foo</a>",
            String::from_utf16(&formatter.finish()).unwrap()
        );
    }

    #[test]
    fn hyperlink_formatting_complex() {
        let d = dom(&[a(&[b(&[tx("foo")]), tx(" "), i(&[tx("bar")])])]);
        let mut formatter = HtmlFormatter::new();
        fmt_node_u16(d.document(), &mut formatter);
        assert_eq!(
            "<a href=\"https://element.io\"><b>foo</b> <i>bar</i></a>",
            String::from_utf16(&formatter.finish()).unwrap()
        );
    }

    #[test]
    fn inline_code_gets_formatted() {
        let d = dom(&[i_c(&[tx("some_code")])]);
        let mut formatter = HtmlFormatter::new();
        fmt_node_u16(d.document(), &mut formatter);
        assert_eq!(
            "<code>some_code</code>",
            String::from_utf16(&formatter.finish()).unwrap()
        );
    }

    #[test]
    fn html_symbols_inside_text_tags_get_escaped() {
        let d = dom(&[tx("<p>Foo & bar</p>")]);
        let mut formatter = HtmlFormatter::new();
        fmt_node_u16(d.document(), &mut formatter);
        assert_eq!(
            "&lt;p&gt;Foo &amp; bar&lt;/p&gt;",
            String::from_utf16(&formatter.finish()).unwrap()
        );
    }

    #[test]
    fn inline_code_text_contents_get_escaped() {
        let d = dom(&[i_c(&[tx("<b>some</b> code")])]);
        let mut formatter = HtmlFormatter::new();
        fmt_node_u16(d.document(), &mut formatter);
        assert_eq!(
            "<code>&lt;b&gt;some&lt;/b&gt; code</code>",
            String::from_utf16(&formatter.finish()).unwrap()
        );
    }

    #[test]
    fn inline_code_node_contents_do_not_get_escaped() {
        let d = dom(&[i_c(&[b(&[tx("some")]), tx(" code")])]);
        let mut formatter = HtmlFormatter::new();
        fmt_node_u16(d.document(), &mut formatter);
        assert_eq!(
            "<code><b>some</b> code</code>",
            String::from_utf16(&formatter.finish()).unwrap()
        );
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
