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

use html5ever::tendril::{StrTendril, TendrilSink};
use html5ever::tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink};
use html5ever::{parse_fragment, Attribute, ExpandedName, QualName};

use super::{
    paqual_name, PaDom, PaDomCreationError, PaDomHandle, PaDomNode,
    PaNodeContainer, PaNodeText,
};

pub(crate) type DomCreationResult = Result<PaDom, PaDomCreationError>;

pub(crate) struct PaDomCreator {
    state: PaDomCreationError,
}

impl PaDomCreator {
    pub fn parse(html: &str) -> DomCreationResult {
        parse_fragment(
            PaDomCreator::default(),
            Default::default(),
            paqual_name(""),
            vec![],
        )
        .from_utf8()
        .one(html.as_bytes())
    }
}

impl Default for PaDomCreator {
    fn default() -> Self {
        Self {
            state: PaDomCreationError::new(),
        }
    }
}

impl TreeSink for PaDomCreator {
    type Handle = PaDomHandle;
    type Output = DomCreationResult;

    fn finish(self) -> Self::Output {
        if self.state.parse_errors.is_empty() {
            Ok(self.state.dom)
        } else {
            Err(self.state)
        }
    }

    fn parse_error(&mut self, msg: std::borrow::Cow<'static, str>) {
        self.state.parse_errors.push(String::from(msg));
    }

    fn get_document(&mut self) -> Self::Handle {
        self.state.dom.document_handle().clone()
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> ExpandedName<'a> {
        self.state.dom.get_node(target).name().expanded()
    }

    fn create_element(
        &mut self,
        name: QualName,
        attrs: Vec<Attribute>,
        flags: ElementFlags,
    ) -> Self::Handle {
        self.state.dom.create_element(name, attrs, flags)
    }

    fn create_comment(&mut self, _text: StrTendril) -> Self::Handle {
        todo!("Comments not yet supported")
    }

    fn create_pi(
        &mut self,
        _target: StrTendril,
        _data: StrTendril,
    ) -> Self::Handle {
        todo!("create_pi not yet supported")
    }

    fn append(
        &mut self,
        parent: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        match child {
            NodeOrText::AppendNode(child) => {
                match self.state.dom.get_mut_node(parent) {
                    PaDomNode::Container(p) => p.children.push(child),
                    PaDomNode::Document(p) => p.children.push(child),
                    PaDomNode::Text(_) => {
                        panic!("Appending node to text! {:?}", parent)
                    }
                }
            }
            NodeOrText::AppendText(tendril) => {
                let text_handle = match self.state.dom.get_node(parent) {
                    PaDomNode::Document(_) => None,
                    PaDomNode::Text(_) => Some(parent.clone()),
                    PaDomNode::Container(PaNodeContainer {
                        children, ..
                    }) => match children
                        .last()
                        .map(|handle| (handle, self.state.dom.get_node(handle)))
                    {
                        Some((last_child_handle, PaDomNode::Text(_))) => {
                            Some(last_child_handle.clone())
                        }
                        _ => None,
                    },
                };

                if let Some(text_handle) = text_handle {
                    if let PaDomNode::Text(p) =
                        self.state.dom.get_mut_node(&text_handle)
                    {
                        p.content += tendril.as_ref();
                    } else {
                        unreachable!(
                            "`text_handle` must map to a `PaDomNode::Text`"
                        )
                    }
                } else {
                    let new_handle =
                        self.state.dom.add_node(PaDomNode::Text(PaNodeText {
                            content: tendril.as_ref().to_owned(),
                        }));

                    match self.state.dom.get_mut_node(parent) {
                        PaDomNode::Container(p) => p.children.push(new_handle),
                        PaDomNode::Document(p) => p.children.push(new_handle),
                        PaDomNode::Text(_) => {
                            panic!("parent changed from container to text!")
                        }
                    }
                }
            }
        };
    }

    fn append_based_on_parent_node(
        &mut self,
        _element: &Self::Handle,
        _prev_element: &Self::Handle,
        _child: NodeOrText<Self::Handle>,
    ) {
        todo!("append_based_on_parent_node not yet supported")
    }

    fn append_doctype_to_document(
        &mut self,
        _name: StrTendril,
        _public_id: StrTendril,
        _system_id: StrTendril,
    ) {
        todo!("append_doctype_to_document not yet supported")
    }

    fn get_template_contents(
        &mut self,
        _target: &Self::Handle,
    ) -> Self::Handle {
        todo!("get_template_contents not yet supported")
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x == y
    }

    fn set_quirks_mode(&mut self, _mode: QuirksMode) {
        // Nothing to do here for now
    }

    fn append_before_sibling(
        &mut self,
        _sibling: &Self::Handle,
        _new_node: NodeOrText<Self::Handle>,
    ) {
        todo!("append_before_sibling not yet supported")
    }

    fn add_attrs_if_missing(
        &mut self,
        target: &Self::Handle,
        attrs: Vec<Attribute>,
    ) {
        let node = self.state.dom.get_mut_node(target);
        if let PaDomNode::Container(node) = node {
            let to_add: Vec<(String, String)> = attrs
                .iter()
                .filter_map(|attr| {
                    let attr_name = attr.name.local.as_ref();
                    if node.attrs.iter().any(|(name, _)| name == attr_name) {
                        Some((
                            attr_name.to_owned(),
                            attr.value.as_ref().to_owned(),
                        ))
                    } else {
                        None
                    }
                })
                .collect();
            node.attrs.extend(to_add);
        } else {
            panic!("Non-element passed to add_attrs_if_missing!");
        }
    }

    fn remove_from_parent(&mut self, _target: &Self::Handle) {
        todo!("remove_from_parent not yet supported")
    }

    fn reparent_children(
        &mut self,
        _node: &Self::Handle,
        _new_parent: &Self::Handle,
    ) {
        todo!("reparent_children not yet supported")
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;
    use crate::dom::parser::{paqual_name, PaDom, PaDomNode, PaNodeContainer};

    trait GarbageCollect {
        fn gc(&mut self);
    }

    impl GarbageCollect for PaDom {
        /// Rebuild the dom removing all unreferenced nodes
        /// Invalidates any existing handles
        fn gc(&mut self) {
            let mut deleted_indices = HashSet::from_iter(0..self.nodes.len());

            fn find_used(
                dom_container: &mut PaDom,
                deleted_indices: &mut HashSet<usize>,
                handle: &PaDomHandle,
            ) {
                deleted_indices.remove(&handle.0);
                let mut children = Vec::new();
                match dom_container.get_node(handle) {
                    PaDomNode::Container(p) => {
                        children.extend(p.children.iter().cloned());
                    }
                    PaDomNode::Document(p) => {
                        children.extend(p.children.iter().cloned());
                    }
                    PaDomNode::Text(_) => {}
                }
                for ch in children {
                    find_used(dom_container, deleted_indices, &ch)
                }
            }

            let document_handle = self.document_handle().clone();
            find_used(self, &mut deleted_indices, &document_handle);

            // Create a new list of nodes with the deleted ones removed
            let mut new_nodes: Vec<PaDomNode> = self
                .nodes
                .iter()
                .enumerate()
                .filter_map(|(i, n)| {
                    if deleted_indices.contains(&i) {
                        None
                    } else {
                        Some(n)
                    }
                })
                .cloned()
                .collect();

            fn remap_handle(
                deleted_indices: &HashSet<usize>,
                handle: &PaDomHandle,
            ) -> PaDomHandle {
                // Every deleted node before this one means this one is
                // reduced by one.
                let mut new_index = handle.0;
                for i in deleted_indices {
                    if *i < handle.0 {
                        new_index -= 1;
                    }
                }
                PaDomHandle(new_index)
            }

            // Modify the handles in all of those nodes to be correct
            for node in &mut new_nodes {
                match node {
                    PaDomNode::Document(n) => {
                        for c in n.children.iter_mut() {
                            *c = remap_handle(&deleted_indices, c);
                        }
                    }
                    PaDomNode::Container(n) => {
                        for c in n.children.iter_mut() {
                            *c = remap_handle(&deleted_indices, c);
                        }
                    }
                    PaDomNode::Text(_) => {}
                }
            }

            let new_document_handle =
                remap_handle(&deleted_indices, &self.document_handle);

            self.nodes = new_nodes;
            self.document_handle = new_document_handle;
        }
    }

    #[derive(Clone, Debug)]
    struct TestNode {
        dom_node: PaDomNode,
        children: Vec<TestNode>,
    }

    fn doc<'a>(children: impl IntoIterator<Item = &'a TestNode>) -> PaDom {
        let mut ret = PaDom::new();

        fn add(
            ret: &mut PaDom,
            parent: &PaDomHandle,
            test_node: TestNode,
        ) -> PaDomHandle {
            let child = ret.add_node(test_node.dom_node);

            let parent = ret.get_mut_node(parent);
            match parent {
                PaDomNode::Container(p) => {
                    p.children.push(child.clone());
                }
                PaDomNode::Document(p) => {
                    p.children.push(child.clone());
                }
                PaDomNode::Text(_) => panic!("Parent can't be a text node"),
            }

            for ch in test_node.children {
                add(ret, &child, ch);
            }

            child
        }

        let document_handle = ret.document_handle().clone();
        for ch in children.into_iter() {
            add(&mut ret, &document_handle, ch.clone());
        }

        ret
    }

    fn el<'a>(
        name: &str,
        children: impl IntoIterator<Item = &'a TestNode>,
    ) -> TestNode {
        TestNode {
            dom_node: PaDomNode::Container(PaNodeContainer {
                name: paqual_name(name),
                attrs: Vec::new(),
                children: Vec::new(),
            }),
            children: children.into_iter().cloned().collect(),
        }
    }

    fn el_attr<'a>(
        name: &str,
        attrs: impl IntoIterator<Item = &'a (&'a str, &'a str)>,
        children: impl IntoIterator<Item = &'a TestNode>,
    ) -> TestNode {
        TestNode {
            dom_node: PaDomNode::Container(PaNodeContainer {
                name: paqual_name(name),
                attrs: attrs
                    .into_iter()
                    .map(|&(n, v)| (n.to_owned(), v.to_owned()))
                    .collect(),
                children: Vec::new(),
            }),
            children: children.into_iter().cloned().collect(),
        }
    }

    fn tx(content: &str) -> TestNode {
        TestNode {
            dom_node: PaDomNode::Text(PaNodeText {
                content: content.to_owned(),
            }),
            children: Vec::new(),
        }
    }

    fn d(mut node: PaDom) -> String {
        node.gc();
        format!("{:?}", node)
    }

    fn parse(input: &str) -> PaDom {
        PaDomCreator::parse(input).unwrap()
    }

    #[test]
    fn parsing_an_empty_string_creates_an_empty_dom() {
        assert_eq!(d(parse("")), d(doc(&[el("html", &[])])));
    }

    #[test]
    fn parsing_a_text_snippet_creates_one_node() {
        assert_eq!(d(parse("foo")), d(doc(&[el("html", &[tx("foo")])])));
    }

    #[test]
    fn parsing_a_tag_creates_a_tag() {
        assert_eq!(d(parse("<i></i>")), d(doc(&[el("html", &[el("i", &[])])])));
    }

    #[test]
    fn parsing_two_tags_creates_two_tags() {
        assert_eq!(
            d(parse("<i></i><b></b>")),
            d(doc(&[el("html", &[el("i", &[]), el("b", &[])])]))
        );
    }

    #[test]
    fn parsing_nested_structures_produces_them() {
        assert_eq!(
            d(parse("A<i>B<b>C</b>D</i>E")),
            d(doc(&[el(
                "html",
                &[
                    tx("A"),
                    el("i", &[tx("B"), el("b", &[tx("C")]), tx("D")]),
                    tx("E")
                ]
            )]))
        );
    }

    #[test]
    fn parsing_tags_with_attributes_preserves_them() {
        assert_eq!(
            d(parse("<span class='foo'>txt</span>")),
            d(doc(&[el(
                "html",
                &[el_attr("span", &[("class", "foo")], &[tx("txt")]),]
            )]))
        );
    }

    #[test]
    fn parsing_text_node_with_escaped_html_entities() {
        assert_eq!(
            d(parse("aaa&lt;strong&gt;bbb&lt;/strong&gt;ccc")),
            d(doc(&[el("html", &[tx("aaa<strong>bbb</strong>ccc")])]))
        )
    }

    // Note: more complex tests are in parse, because it's more ergonomic to
    // work with a real Dom instead of PaDom, because it converts back to HTML
    // nicely.
}
