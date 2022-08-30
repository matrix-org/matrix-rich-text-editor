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

use std::fmt::Display;

use crate::dom::nodes::{ContainerNode, ContainerNodeKind, DomNode};
use crate::dom::{
    find_range, to_raw_text::ToRawText, DomHandle, Range, UnicodeString,
};
use crate::ToHtml;

#[derive(Clone, Debug, PartialEq)]
pub struct Dom<S>
where
    S: UnicodeString,
{
    document: DomNode<S>,
}

impl<S> Dom<S>
where
    S: UnicodeString,
{
    pub fn new(top_level_items: Vec<DomNode<S>>) -> Self {
        let mut document = ContainerNode::new(
            S::new(),
            ContainerNodeKind::Generic,
            None,
            top_level_items,
        );
        document.set_handle(DomHandle::from_raw(Vec::new()));

        Self {
            document: DomNode::Container(document),
        }
    }

    pub fn document(&self) -> &ContainerNode<S> {
        // Would be nice if we could avoid this, but it is really convenient
        // in several places to be able to treat document as a DomNode.
        if let DomNode::Container(ret) = &self.document {
            ret
        } else {
            panic!("Document should always be a Container!")
        }
    }

    pub fn document_mut(&mut self) -> &mut ContainerNode<S> {
        // Would be nice if we could avoid this, but it is really convenient
        // in several places to be able to treat document as a DomNode.
        if let DomNode::Container(ret) = &mut self.document {
            ret
        } else {
            panic!("Document should always be a Container!")
        }
    }

    pub fn children(&self) -> &Vec<DomNode<S>> {
        self.document().children()
    }

    pub fn append_child(&mut self, child: DomNode<S>) {
        self.document_mut().append_child(child)
    }

    pub fn replace(&mut self, node_handle: DomHandle, nodes: Vec<DomNode<S>>) {
        let parent_node = self.lookup_node_mut(node_handle.parent_handle());
        let index = node_handle.index_in_parent();
        match parent_node {
            DomNode::Text(_n) => panic!("Text nodes can't have children"),
            DomNode::Container(n) => n.replace_child(index, nodes),
        }
    }

    /// Given the start and end code units, find which nodes of this Dom are
    /// selected. The returned range lists all the Dom nodes involved.
    pub fn find_range(&self, start: usize, end: usize) -> Range {
        find_range::find_range(self, start, end)
    }

    pub(crate) fn document_handle(&self) -> DomHandle {
        self.document.handle()
    }

    pub fn find_parent_list_item(
        &mut self,
        child_handle: DomHandle,
    ) -> Option<DomHandle> {
        fn find_list_item<S>(
            dom: &Dom<S>,
            node: &DomNode<S>,
        ) -> Option<DomHandle>
        where
            S: UnicodeString,
        {
            if !node.handle().has_parent() {
                return None;
            }
            let parent_handle = node.handle().parent_handle();
            let parent_node = dom.lookup_node(parent_handle.clone());

            match parent_node {
                DomNode::Text(_) => find_list_item(dom, parent_node),
                DomNode::Container(n) => {
                    if n.is_list_item() {
                        Some(parent_handle)
                    } else {
                        find_list_item(dom, parent_node)
                    }
                }
            }
        }

        find_list_item(self, self.lookup_node(child_handle))
    }

    /// Find the node based on its handle.
    /// Panics if the handle is unset or invalid
    pub fn lookup_node(&self, node_handle: DomHandle) -> &DomNode<S> {
        // TODO: consider taking a reference to handle to avoid clones
        fn nth_child<S>(element: &ContainerNode<S>, idx: usize) -> &DomNode<S>
        where
            S: UnicodeString,
        {
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
    ) -> &mut DomNode<S> {
        // TODO: horrible that we repeat lookup_node's logic. Can we share?
        fn nth_child<S>(
            element: &mut ContainerNode<S>,
            idx: usize,
        ) -> &mut DomNode<S>
        where
            S: UnicodeString,
        {
            element.get_child_mut(idx).expect(
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

    // Return the number of code points in the string representation of this
    // Dom.
    pub fn text_len(&self) -> usize {
        self.document.text_len()
    }
}

impl<S> ToHtml<S> for Dom<S>
where
    S: UnicodeString,
{
    fn fmt_html(&self, f: &mut super::HtmlFormatter<S>) {
        self.document.fmt_html(f)
    }
}

impl<S> ToRawText<S> for Dom<S>
where
    S: UnicodeString,
{
    fn to_raw_text(&self) -> S {
        self.document.to_raw_text()
    }
}

impl<S> Display for Dom<S>
where
    S: UnicodeString,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_html().to_utf8())
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
    use crate::dom::nodes::TextNode;
    use crate::tests::testutils_composer_model::cm;
    use crate::tests::testutils_conversion::utf16;
    use crate::tests::testutils_dom::{a, b, dom, i, i_c, tn};

    // Creation and handles

    #[test]
    fn can_create_a_dom_and_add_nodes() {
        // Create a simple DOM
        let dom = Dom::new(vec![
            DomNode::Text(TextNode::from(utf16("a"))),
            DomNode::new_formatting(
                utf16("b"),
                vec![DomNode::Text(TextNode::from(utf16("b")))],
            ),
        ]);

        // The DOM was created successfully
        assert_eq!(dom.to_string(), "a<b>b</b>");
    }

    #[test]
    fn can_find_toplevel_nodes_via_handles() {
        // Create a simple DOM
        let dom = Dom::new(vec![
            DomNode::Text(TextNode::from(utf16("a"))),
            DomNode::new_formatting(
                utf16("b"),
                vec![DomNode::Text(TextNode::from(utf16("b")))],
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
            tn("foo"),
            b(&[tn("BOLD"), b(&[tn("uberbold")])]),
            tn("bar"),
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
        let mut dom = dom(&[tn("foo"), tn("bar")]);

        let node_handle = dom.children()[0].handle();
        let inserted_nodes = vec![tn("ab"), b(&[tn("cd")]), tn("ef")];

        dom.replace(node_handle, inserted_nodes);

        // Node is replaced by new insertion
        assert_eq!(dom.to_string(), "ab<b>cd</b>efbar");
        // Subsequent node handle is properly updated
        let bar_node = &dom.children()[3];
        assert_eq!(bar_node.handle().index_in_parent(), 3);
    }

    #[test]
    fn can_replace_deep_node_with_multiple_nodes() {
        let mut dom = dom(&[b(&[tn("foo")])]);

        let node_handle = kids(&dom.children()[0])[0].handle();
        let inserted_nodes = vec![tn("f"), i(&[tn("o")]), tn("o")];

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
        assert_eq!(dom(&[tn("foo")]).to_string(), "foo");
    }

    #[test]
    fn mixed_text_and_tags_serialises() {
        assert_eq!(
            dom(&[tn("foo"), b(&[tn("BOLD")]), tn("bar")]).to_string(),
            "foo<b>BOLD</b>bar"
        );
    }

    #[test]
    fn nested_tags_serialise() {
        assert_eq!(
            dom(&[
                tn("foo"),
                b(&[tn("BO"), i(&[tn("LD")])]),
                i(&[tn("it")]),
                tn("bar")
            ])
            .to_string(),
            "foo<b>BO<i>LD</i></b><i>it</i>bar"
        );
    }

    #[test]
    fn empty_tag_serialises() {
        assert_eq!(dom(&[b(&[]),]).to_string(), "<b></b>");
    }

    #[test]
    fn hyperlink_formatting_simple() {
        let d = dom(&[a(&[tn("foo")])]);
        assert_eq!(
            "<a href=\"https://element.io\">foo</a>",
            d.to_html().to_utf8()
        );
    }

    #[test]
    fn hyperlink_formatting_complex() {
        let d = dom(&[a(&[b(&[tn("foo")]), tn(" "), i(&[tn("bar")])])]);
        assert_eq!(
            "<a href=\"https://element.io\"><b>foo</b> <i>bar</i></a>",
            d.to_html().to_utf8()
        );
    }

    #[test]
    fn inline_code_gets_formatted() {
        let d = dom(&[i_c(&[tn("some_code")])]);
        assert_eq!("<code>some_code</code>", d.to_html().to_utf8());
    }

    #[test]
    fn html_symbols_inside_text_tags_get_escaped() {
        let d = dom(&[tn("<p>Foo & bar</p>")]);
        assert_eq!("&lt;p&gt;Foo &amp; bar&lt;/p&gt;", d.to_html().to_utf8());
    }

    #[test]
    fn inline_code_text_contents_get_escaped() {
        let d = dom(&[i_c(&[tn("<b>some</b> code")])]);
        assert_eq!(
            "<code>&lt;b&gt;some&lt;/b&gt; code</code>",
            d.to_html().to_utf8()
        );
    }

    #[test]
    fn inline_code_node_contents_do_not_get_escaped() {
        let d = dom(&[i_c(&[b(&[tn("some")]), tn(" code")])]);
        assert_eq!("<code><b>some</b> code</code>", d.to_html().to_utf8());
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

    /// If this node is an element, return its children - otherwise panic
    fn kids<S>(node: &DomNode<S>) -> &Vec<DomNode<S>>
    where
        S: UnicodeString,
    {
        match node {
            DomNode::Container(n) => n.children(),
            DomNode::Text(_) => {
                panic!("We expected an Element, but found Text")
            }
        }
    }

    #[test]
    fn text_len_ignores_html_tags() {
        assert_eq!(0, cm("|").state.dom.text_len());
        assert_eq!(2, cm("aa|").state.dom.text_len());
        assert_eq!(2, cm("a<b>a|</b>").state.dom.text_len());
        assert_eq!(2, cm("<del><i>a</i><b>a|</b></del>").state.dom.text_len());
    }
}
