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

use crate::dom::dom_creation_error::HtmlParseError;
use crate::dom::nodes::dom_node::DomNodeKind::CodeBlock;
use crate::dom::nodes::ContainerNode;
use crate::dom::Dom;
use crate::{DomHandle, DomNode, UnicodeString};

pub fn parse<S>(html: &str) -> Result<Dom<S>, HtmlParseError>
where
    S: UnicodeString,
{
    cfg_if::cfg_if! {
        if #[cfg(feature = "sys")] {
            sys::HtmlParser::default().parse(html)
        } else if #[cfg(all(feature = "js", target_arch = "wasm32"))] {
            js::HtmlParser::default().parse(html)
        } else {
            unreachable!("The `sys` or `js` are mutually exclusive, and one of them must be enabled.")
        }
    }
}

#[cfg(feature = "sys")]
mod sys {
    use super::super::padom_node::PaDomNode;
    use super::super::PaNodeContainer;
    use super::super::{PaDom, PaDomCreationError, PaDomCreator};
    use super::*;
    use crate::dom::nodes::dom_node::DomNodeKind;
    use crate::dom::nodes::dom_node::DomNodeKind::CodeBlock;
    use crate::dom::nodes::{ContainerNode, DomNode};
    use crate::ListType;

    pub(super) struct HtmlParser {
        current_path: Vec<DomNodeKind>,
    }
    impl HtmlParser {
        pub(super) fn default() -> Self {
            Self {
                current_path: Vec::new(),
            }
        }

        pub(super) fn parse<S>(
            &mut self,
            html: &str,
        ) -> Result<Dom<S>, HtmlParseError>
        where
            S: UnicodeString,
        {
            PaDomCreator::parse(html)
                .map(|pa_dom| {
                    let dom = self.padom_to_dom(pa_dom);
                    post_process_code_blocks(dom)
                })
                .map_err(|err| {
                    self.padom_creation_error_to_html_parse_error(err)
                })
        }

        /// Convert a [PaDom] into a [Dom].
        ///
        /// [PaDom] is purely used within the parsing process (using html5ever) - in it,
        /// parents refer to their children by handles, and all the nodes are owned in
        /// a big list held by the PaDom itself. PaDoms may also contain garbage nodes
        /// that were created during parsing but are no longer needed. A garbage
        /// collection method was written for testing and is inside padom_creator's
        /// test code. The conversion process here ignores garbage nodes, so they do
        /// not appear in the final Dom.
        ///
        /// [Dom] is for general use. Parent nodes own their children, and Dom may be
        /// cloned, compared, and converted into an HTML string.
        fn padom_to_dom<S>(&mut self, padom: PaDom) -> Dom<S>
        where
            S: UnicodeString,
        {
            let mut ret = Dom::new(Vec::new());
            let doc = ret.document_mut();

            if let PaDomNode::Document(padoc) = padom.get_document() {
                self.convert(&padom, padoc, doc)
            } else {
                panic!("Document was not a document!");
            }
            ret
        }

        /// Copy all panode's information into node.
        fn convert<S>(
            &mut self,
            padom: &PaDom,
            panode: &PaNodeContainer,
            node: &mut ContainerNode<S>,
        ) where
            S: UnicodeString,
        {
            for child_handle in &panode.children {
                let child = padom.get_node(child_handle);
                match child {
                    PaDomNode::Container(child) => {
                        self.convert_container(padom, child, node);
                    }
                    PaDomNode::Document(_) => {
                        panic!("Found a document inside a document!")
                    }
                    PaDomNode::Text(text) => {
                        // Special case for code block, translate '\n' into <br /> nodes
                        let is_inside_code_block =
                            self.current_path.contains(&CodeBlock);
                        convert_text(&text.content, node, is_inside_code_block);
                    }
                }
            }
        }

        /// Copy all panode's information into node (now we know it's a container).
        fn convert_container<S>(
            &mut self,
            padom: &PaDom,
            child: &PaNodeContainer,
            node: &mut ContainerNode<S>,
        ) where
            S: UnicodeString,
        {
            let cur_path_idx = self.current_path.len();
            let tag = child.name.local.as_ref();
            match tag {
                "b" | "code" | "del" | "em" | "i" | "strong" | "u" => {
                    let formatting_node = Self::new_formatting(tag);
                    if tag == "code" && self.current_path.contains(&CodeBlock) {
                        self.convert_children(padom, child, Some(node));
                    } else {
                        self.current_path.push(formatting_node.kind());
                        node.append_child(formatting_node);
                        self.convert_children(
                            padom,
                            child,
                            last_container_mut_in(node),
                        );
                        self.current_path.remove(cur_path_idx);
                    }
                }
                "br" => {
                    node.append_child(Self::new_line_break());
                }
                "ol" | "ul" => {
                    self.current_path.push(DomNodeKind::List);
                    node.append_child(Self::new_list(tag));
                    self.convert_children(
                        padom,
                        child,
                        last_container_mut_in(node),
                    );
                    self.current_path.remove(cur_path_idx);
                }
                "li" => {
                    self.current_path.push(DomNodeKind::ListItem);
                    node.append_child(Self::new_list_item());
                    self.convert_children(
                        padom,
                        child,
                        last_container_mut_in(node),
                    );
                    self.current_path.remove(cur_path_idx);
                }
                "a" => {
                    self.current_path.push(DomNodeKind::Link);
                    node.append_child(Self::new_link(child));
                    self.convert_children(
                        padom,
                        child,
                        last_container_mut_in(node),
                    );
                    self.current_path.remove(cur_path_idx);
                }
                "pre" => {
                    self.current_path.push(DomNodeKind::CodeBlock);
                    node.append_child(Self::new_code_block());
                    self.convert_children(
                        padom,
                        child,
                        last_container_mut_in(node),
                    );
                    self.current_path.remove(cur_path_idx);
                }
                "blockquote" => {
                    self.current_path.push(DomNodeKind::Quote);
                    node.append_child(Self::new_quote());
                    self.convert_children(
                        padom,
                        child,
                        last_container_mut_in(node),
                    );
                    self.current_path.remove(cur_path_idx);
                }
                "html" => {
                    // Skip the html tag - add its children to the
                    // current node directly.
                    self.convert(padom, child, node);
                }
                "p" => {
                    self.current_path.push(DomNodeKind::Paragraph);
                    node.append_child(Self::new_paragraph());
                    self.convert_children(
                        padom,
                        child,
                        last_container_mut_in(node),
                    );
                    self.current_path.remove(cur_path_idx);
                }
                _ => {
                    // Ignore tags we don't recognise
                    // We should log - see internal task PSU-741
                }
            };
        }

        /// Recurse into panode's children and convert them too
        fn convert_children<S>(
            &mut self,
            padom: &PaDom,
            child: &PaNodeContainer,
            new_node: Option<&mut ContainerNode<S>>,
        ) where
            S: UnicodeString,
        {
            if let Some(new_node) = new_node {
                self.convert(padom, child, new_node);
            } else {
                panic!("Container became non-container!");
            }
        }

        /// Create a formatting node
        fn new_formatting<S>(tag: &str) -> DomNode<S>
        where
            S: UnicodeString,
        {
            DomNode::Container(
                ContainerNode::new_formatting_from_tag(tag.into(), Vec::new())
                    .unwrap_or_else(|| panic!("Unknown format tag {}", tag)),
            )
        }

        /// Create a br node
        fn new_line_break<S>() -> DomNode<S>
        where
            S: UnicodeString,
        {
            DomNode::new_line_break()
        }

        /// Create a link node
        fn new_link<S>(child: &PaNodeContainer) -> DomNode<S>
        where
            S: UnicodeString,
        {
            // initial implementation, firstly check if we have either `contenteditable=false` or `data-mention-type=`
            // attributes, if so then we're going to add a mention instead of a link
            // TODO move to inferring link or mention from href when utils exist
            let is_mention = child.attrs.iter().any(|(k, v)| {
                k == &String::from("contenteditable")
                    && v == &String::from("false")
                    || k == &String::from("data-mention-type")
            });

            if is_mention {
                // if we have a mention, filtering out the href and contenteditable attributes because
                // we add these attributes when creating the mention and don't want repetition
                let attributes = child
                    .attrs
                    .iter()
                    .filter(|(k, _)| {
                        k != &String::from("href")
                            && k != &String::from("contenteditable")
                    })
                    .map(|(k, v)| (k.as_str().into(), v.as_str().into()))
                    .collect();
                DomNode::Container(ContainerNode::new_mention(
                    child.get_attr("href").unwrap_or("").into(),
                    Vec::new(),
                    attributes,
                ))
            } else {
                DomNode::Container(ContainerNode::new_link(
                    child.get_attr("href").unwrap_or("").into(),
                    Vec::new(),
                ))
            }
        }

        /// Create a list node
        fn new_list<S>(tag: &str) -> DomNode<S>
        where
            S: UnicodeString,
        {
            DomNode::Container(ContainerNode::new_list(
                ListType::try_from(S::from(tag)).unwrap(),
                Vec::new(),
            ))
        }

        /// Create a list item node
        fn new_list_item<S>() -> DomNode<S>
        where
            S: UnicodeString,
        {
            DomNode::Container(ContainerNode::new_list_item(Vec::new()))
        }

        /// Create a code block node
        fn new_code_block<S>() -> DomNode<S>
        where
            S: UnicodeString,
        {
            DomNode::Container(ContainerNode::new_code_block(Vec::new()))
        }

        /// Create a quote node
        fn new_quote<S>() -> DomNode<S>
        where
            S: UnicodeString,
        {
            DomNode::Container(ContainerNode::new_quote(Vec::new()))
        }

        /// Create a paragraph
        fn new_paragraph<S>() -> DomNode<S>
        where
            S: UnicodeString,
        {
            DomNode::Container(ContainerNode::new_paragraph(Vec::new()))
        }

        fn padom_creation_error_to_html_parse_error(
            &mut self,
            e: PaDomCreationError,
        ) -> HtmlParseError {
            HtmlParseError {
                parse_errors: e.parse_errors,
            }
        }
    }

    #[cfg(test)]
    mod test {
        use crate::dom::parser::parse::sys::HtmlParser;
        use crate::dom::Dom;
        use indoc::indoc;
        use speculoos::{assert_that, AssertionFailure, Spec};
        use widestring::Utf16String;

        use crate::tests::testutils_composer_model::restore_whitespace;
        use crate::{ToHtml, ToTree};

        use super::*;

        trait Roundtrips<T> {
            fn roundtrips(&self);
        }

        impl<'s, T> Roundtrips<T> for Spec<'s, T>
        where
            T: AsRef<str>,
        {
            fn roundtrips(&self) {
                let subject = self.subject.as_ref();
                let dom = parse::<Utf16String>(subject).unwrap();

                // After parsing all our invariants should be satisifed
                dom.explicitly_assert_invariants();

                let output = restore_whitespace(&dom.to_html().to_string());
                if output != subject {
                    AssertionFailure::from_spec(self)
                        .with_expected(String::from(subject))
                        .with_actual(output)
                        .fail();
                }
            }
        }

        #[test]
        fn parse_plain_text() {
            assert_that!("some text").roundtrips();
        }

        #[test]
        fn parse_simple_tag() {
            assert_that!("<strong>sdfds</strong>").roundtrips();
        }

        #[test]
        fn parse_tag_with_surrounding_text() {
            assert_that!("before <strong> within </strong> after").roundtrips();
            assert_that!("before<strong>within</strong>after").roundtrips();
        }

        #[test]
        fn parse_nested_tags() {
            assert_that!("<b><em>ZZ</em></b>").roundtrips();
            assert_that!("X<b>Y<em>ZZ</em>0</b>1").roundtrips();
            assert_that!(" X <b> Y <em> ZZ </em> 0 </b> 1 ").roundtrips();
        }

        #[test]
        fn parse_tags_with_attributes() {
            assert_that!(r#"<b><a href="http://example.com">ZZ</a></b>"#)
                .roundtrips();
        }

        #[test]
        fn parse_br_tag() {
            assert_that!("<br />").roundtrips();
        }

        #[test]
        fn parse_code_block_keeps_internal_code_tag() {
            let html = "\
                <p>foo</p>\
                <pre><code>Some code</code></pre>\
                <p>bar</p>";
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                ├>p
                │ └>"foo"
                ├>codeblock
                │ └>p
                │   └>"Some code"
                └>p
                  └>"bar"
                "#}
            );
        }

        #[test]
        fn parse_code_block_roundtrips() {
            assert_that!(
                "<p>foo</p><pre><code>Some code</code></pre><p>bar</p>"
            )
            .roundtrips();
        }

        #[test]
        fn parse_code_block_post_processes_it() {
            let mut parser = HtmlParser::default();
            let html = "<pre><code><b>Test\nCode</b></code></pre>";
            let dom: Dom<Utf16String> = PaDomCreator::parse(html)
                .map(|pa_dom| parser.padom_to_dom(pa_dom))
                .ok()
                .unwrap();
            // First, line breaks are added as placeholders for paragraphs
            assert_eq!(
                dom.to_html().to_string(),
                "<pre><code><b>Test<br />Code</b></code></pre>"
            );
            // Then these line breaks are post-processed and we get the actual paragraphs
            let dom = post_process_code_blocks_lines(
                dom,
                &DomHandle::from_raw(vec![0]),
            );
            assert_eq!(
                dom.to_html().to_string(),
                "<pre><code><b>Test</b>\n<b>Code</b></code></pre>"
            );
        }

        #[test]
        fn parse_quote() {
            assert_that!(
                "<p>foo</p><blockquote>A quote</blockquote><p>bar</p>"
            )
            .roundtrips();
        }

        #[test]
        fn parse_paragraph() {
            assert_that!("<p>foo</p><p>A paragraph</p><p>bar</p>").roundtrips();
        }

        #[test]
        fn nbsp_chars_are_removed() {
            let html = "\
                <p>\u{A0}</p>\
                <pre><code>\u{A0}\n\u{A0}</code></pre>\
                <p>\u{A0}</p>";
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                ├>p
                ├>codeblock
                │ ├>p
                │ └>p
                └>p
                "#}
            );
        }

        #[test]
        fn nbsp_text_is_removed() {
            let html = "\
                <p>&nbsp;</p>\
                <pre><code>&nbsp;\n&nbsp;</code></pre>\
                <p>&nbsp;</p>";
            let dom: Dom<Utf16String> =
                HtmlParser::default().parse(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                ├>p
                ├>codeblock
                │ ├>p
                │ └>p
                └>p
                "#}
            );
        }
    }
}

fn post_process_code_blocks<S: UnicodeString>(mut dom: Dom<S>) -> Dom<S> {
    let code_block_handles = find_code_block_handles(&dom);
    for handle in code_block_handles.iter().rev() {
        dom = post_process_code_blocks_lines(dom, handle);
    }
    dom
}

fn find_code_block_handles<S: UnicodeString>(dom: &Dom<S>) -> Vec<DomHandle> {
    dom.iter()
        .filter(|n| n.kind() == CodeBlock)
        .map(|n| n.handle())
        .collect()
}

fn post_process_code_blocks_lines<S: UnicodeString>(
    mut dom: Dom<S>,
    handle: &DomHandle,
) -> Dom<S> {
    assert_eq!(dom.lookup_node(handle).kind(), CodeBlock);
    let last_handle = dom.last_node_handle_in_sub_tree(handle);
    let mut next_handle = last_handle.clone();
    let mut children = Vec::new();
    let mut line_break_handles = Vec::new();
    for node in dom.iter_from_handle(&last_handle).rev() {
        if node.is_line_break() || node.handle() == *handle {
            if node.handle() == next_handle {
                line_break_handles.push(next_handle.next_sibling());
            } else {
                line_break_handles.push(next_handle.clone());
            }
        }
        next_handle = node.handle();
        if node.handle().depth() <= handle.depth() {
            break;
        }
    }

    for line_break_handle in line_break_handles {
        let mut sub_tree =
            dom.split_sub_tree_from(&line_break_handle, 0, handle.depth());
        if line_break_handle.index_in_parent() > 0 {
            // Remove line break too
            dom.remove(&line_break_handle.prev_sibling());
        }
        let node =
            DomNode::new_paragraph(sub_tree.document_mut().remove_children());
        children.insert(0, node);
    }

    let needs_removal = if dom.contains(handle) {
        let block = dom.lookup_node(handle);
        block.kind() == CodeBlock && block.is_empty()
    } else {
        false
    };
    if needs_removal {
        dom.remove(handle);
    }

    dom.insert_at(handle, DomNode::new_code_block(children));
    dom
}

#[cfg(feature = "sys")]
fn last_container_mut_in<S: UnicodeString>(
    node: &mut ContainerNode<S>,
) -> Option<&mut ContainerNode<S>> {
    node.last_child_mut().and_then(|n| n.as_container_mut())
}

fn convert_text<S: UnicodeString>(
    text: &str,
    node: &mut ContainerNode<S>,
    is_inside_code_block: bool,
) {
    if is_inside_code_block {
        let text_nodes: Vec<_> = text.split('\n').collect();
        let text_nodes_len = text_nodes.len();
        for (i, str) in text_nodes.into_iter().enumerate() {
            let is_nbsp = str == "\u{A0}" || str == "&nbsp;";
            if !str.is_empty() && !is_nbsp {
                let text_node = DomNode::new_text(str.into());
                node.append_child(text_node);
            }
            if i + 1 < text_nodes_len {
                node.append_child(DomNode::new_line_break());
            }
        }
    } else {
        let contents = text;
        let is_nbsp = contents == "\u{A0}" || contents == "&nbsp;";
        if !is_nbsp {
            node.append_child(DomNode::new_text(contents.into()));
        }
    }
}

#[cfg(all(feature = "js", target_arch = "wasm32"))]
mod js {
    use super::*;
    use crate::dom::nodes::dom_node::DomNodeKind;
    use crate::{
        dom::nodes::{ContainerNode, DomNode},
        InlineFormatType, ListType,
    };
    use std::fmt;
    use wasm_bindgen::JsCast;
    use web_sys::{Document, DomParser, Element, NodeList, SupportedType};

    pub(super) struct HtmlParser {
        current_path: Vec<DomNodeKind>,
    }
    impl HtmlParser {
        pub(super) fn default() -> Self {
            Self {
                current_path: Vec::new(),
            }
        }

        pub(super) fn parse<S>(
            &mut self,
            html: &str,
        ) -> Result<Dom<S>, HtmlParseError>
        where
            S: UnicodeString,
        {
            let parser: DomParser = DomParser::new().map_err(|_| {
                to_dom_creation_error(
                    "Failed to create the `DOMParser` from JavaScript",
                )
            })?;

            let document = parser
                .parse_from_string(html, SupportedType::TextHtml)
                .map_err(|_| {
                    to_dom_creation_error(
                        "Failed to convert the Web `Document` to internal `Dom`",
                    )
                })?;

            self.webdom_to_dom(document).map_err(to_dom_creation_error)
        }

        fn webdom_to_dom<S>(
            &mut self,
            webdoc: Document,
        ) -> Result<Dom<S>, Error>
        where
            S: UnicodeString,
        {
            let body = webdoc.body().ok_or_else(|| Error::NoBody)?;
            self.convert(body.child_nodes())
        }

        fn convert<S>(&mut self, nodes: NodeList) -> Result<Dom<S>, Error>
        where
            S: UnicodeString,
        {
            let number_of_nodes = nodes.length() as usize;
            let mut dom = Dom::new(Vec::with_capacity(number_of_nodes));
            let dom_document = dom.document_mut();

            self.convert_container(nodes, dom_document)?;

            dom = post_process_code_blocks(dom);

            Ok(dom)
        }

        fn convert_container<S>(
            &mut self,
            nodes: NodeList,
            dom: &mut ContainerNode<S>,
        ) -> Result<(), Error>
        where
            S: UnicodeString,
        {
            let number_of_nodes = nodes.length() as usize;

            for nth in 0..number_of_nodes {
                let node = nodes.get(nth as _).unwrap();

                match node.node_name().as_str() {
                    "BR" => {
                        dom.append_child(DomNode::new_line_break());
                    }

                    "#text" => match node.node_value() {
                        Some(value) => {
                            let is_inside_code_block =
                                self.current_path.contains(&CodeBlock);
                            convert_text(
                                value.as_str(),
                                dom,
                                is_inside_code_block,
                            );
                        }
                        _ => {}
                    },

                    "A" => {
                        self.current_path.push(DomNodeKind::Link);

                        // TODO move to inferring link or mention from href when utils exist
                        let is_mention = node
                            .unchecked_ref::<Element>()
                            .has_attribute("data-mention-type");

                        let mut attributes = vec![];
                        let valid_attributes = ["data-mention-type", "style"];

                        for attr in valid_attributes.into_iter() {
                            if node
                                .unchecked_ref::<Element>()
                                .has_attribute(attr)
                            {
                                attributes.push((
                                    attr.into(),
                                    node.unchecked_ref::<Element>()
                                        .get_attribute(attr)
                                        .unwrap_or_default()
                                        .into(),
                                ))
                            }
                        }

                        let url = node
                            .unchecked_ref::<Element>()
                            .get_attribute("href")
                            .unwrap_or_default()
                            .into();
                        let children =
                            self.convert(node.child_nodes())?.take_children();

                        if is_mention {
                            dom.append_child(DomNode::new_mention(
                                url, children, attributes,
                            ));
                        } else {
                            dom.append_child(DomNode::new_link(url, children));
                        }

                        self.current_path.pop();
                    }

                    "OL" => {
                        self.current_path.push(DomNodeKind::List);
                        dom.append_child(DomNode::Container(
                            ContainerNode::new_list(
                                ListType::Ordered,
                                self.convert(node.child_nodes())?
                                    .take_children(),
                            ),
                        ));
                        self.current_path.pop();
                    }

                    "UL" => {
                        self.current_path.push(DomNodeKind::List);
                        dom.append_child(DomNode::Container(
                            ContainerNode::new_list(
                                ListType::Unordered,
                                self.convert(node.child_nodes())?
                                    .take_children(),
                            ),
                        ));
                        self.current_path.pop();
                    }

                    "LI" => {
                        self.current_path.push(DomNodeKind::ListItem);
                        dom.append_child(DomNode::Container(
                            ContainerNode::new_list_item(
                                self.convert(node.child_nodes())?
                                    .take_children(),
                            ),
                        ));
                        self.current_path.pop();
                    }

                    "PRE" => {
                        self.current_path.push(DomNodeKind::CodeBlock);
                        let children = node.child_nodes();
                        let children = if children.length() == 1
                            && children.get(0).unwrap().node_name().as_str()
                                == "CODE"
                        {
                            let code_node = children.get(0).unwrap();
                            code_node.child_nodes()
                        } else {
                            children
                        };
                        dom.append_child(DomNode::Container(
                            ContainerNode::new_code_block(
                                self.convert(children)?.take_children(),
                            ),
                        ));
                        self.current_path.pop();
                    }

                    "BLOCKQUOTE" => {
                        self.current_path.push(DomNodeKind::Quote);
                        dom.append_child(DomNode::Container(
                            ContainerNode::new_quote(
                                self.convert(node.child_nodes())?
                                    .take_children(),
                            ),
                        ));
                        self.current_path.pop();
                    }

                    "P" => {
                        self.current_path.push(DomNodeKind::Paragraph);
                        dom.append_child(DomNode::Container(
                            ContainerNode::new_paragraph(
                                self.convert(node.child_nodes())?
                                    .take_children(),
                            ),
                        ));
                        self.current_path.pop();
                    }

                    node_name => {
                        let children_nodes =
                            self.convert(node.child_nodes())?.take_children();

                        let formatting_kind = match node_name {
                            "STRONG" | "B" => InlineFormatType::Bold,
                            "EM" | "I" => InlineFormatType::Italic,
                            "DEL" => InlineFormatType::StrikeThrough,
                            "U" => InlineFormatType::Underline,
                            "CODE" => InlineFormatType::InlineCode,
                            _ => {
                                return Err(Error::UnknownNode(
                                    node_name.to_owned(),
                                ))
                            }
                        };

                        self.current_path.push(DomNodeKind::Formatting(
                            formatting_kind.clone(),
                        ));

                        dom.append_child(DomNode::Container(
                            ContainerNode::new_formatting(
                                formatting_kind,
                                children_nodes,
                            ),
                        ));
                        self.current_path.pop();
                    }
                }
            }

            Ok(())
        }
    }

    fn to_dom_creation_error<E>(error: E) -> HtmlParseError
    where
        E: ToString,
    {
        HtmlParseError {
            parse_errors: vec![error.to_string()],
        }
    }

    enum Error {
        NoBody,
        UnknownNode(String),
    }

    impl fmt::Display for Error {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::NoBody => {
                    write!(
                        formatter,
                        "The `Document` does not have a `<body>` element"
                    )
                }

                Self::UnknownNode(node_name) => {
                    write!(formatter, "Node `{node_name}` is not supported")
                }
            }
        }
    }

    #[cfg(all(test, target_arch = "wasm32"))]
    mod tests {
        use super::*;
        use crate::{
            tests::testutils_composer_model::restore_whitespace, ToHtml, ToTree,
        };
        use indoc::indoc;
        use wasm_bindgen_test::*;
        use widestring::Utf16String;

        wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

        fn roundtrip(html: &str) {
            let parse = HtmlParser::default().parse::<Utf16String>(html);

            assert!(
                parse.is_ok(),
                "Failed to parse the following HTML fragment: `{html}`"
            );

            let dom = parse.unwrap();
            let html_again = restore_whitespace(&dom.to_html().to_string());

            assert_eq!(html, html_again);
        }

        #[wasm_bindgen_test]
        fn formatting() {
            roundtrip("foo <strong>bar</strong> baz");
            roundtrip("foo <em>bar</em> baz");
            roundtrip("foo <del>bar</del> baz");
            roundtrip("foo <u>bar</u> baz");
            roundtrip("foo <code>bar</code> baz");
        }

        #[wasm_bindgen_test]
        fn br() {
            roundtrip("foo<br />bar");
        }

        #[wasm_bindgen_test]
        fn a() {
            roundtrip(r#"foo <a href="url">bar</a> baz"#);
            roundtrip(r#"foo <a href="">bar</a> baz"#);
        }

        #[wasm_bindgen_test]
        fn a_with_attributes() {
            roundtrip(
                r#"<a data-mention-type="user" style="something" href="http://example.com" contenteditable="false">a user mention</a>"#,
            );
        }

        #[wasm_bindgen_test]
        fn a_with_bad_attribute() {
            let html = r#"<a invalidattribute="true" href="http://example.com">a user mention</a>"#;
            let dom = HtmlParser::default().parse::<Utf16String>(html).unwrap();
            assert_eq!(
                dom.to_string(),
                r#"<a href="http://example.com">a user mention</a>"#
            );
        }

        #[wasm_bindgen_test]
        fn ul() {
            roundtrip("foo <ul><li>item1</li><li>item2</li></ul> bar");
        }

        #[wasm_bindgen_test]
        fn ol() {
            roundtrip("foo <ol><li>item1</li><li>item2</li></ol> bar");
        }

        #[wasm_bindgen_test]
        fn pre() {
            roundtrip("foo <pre><code>~Some code</code></pre> bar");
        }

        #[wasm_bindgen_test]
        fn paragraph() {
            roundtrip("<p>foo</p><p>Text</p><p>bar</p>");
        }

        #[wasm_bindgen_test]
        fn pre_removes_internal_code() {
            let html = "<p>foo</p><pre><code>Some code</code></pre><p>bar</p>";
            let dom = HtmlParser::default().parse::<Utf16String>(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
        
                ├>p
                │ └>"foo"
                ├>codeblock
                │ └>p
                │   └>"Some code"
                └>p
                  └>"bar"
                "#}
            );
        }

        #[wasm_bindgen_test]
        fn blockquote() {
            roundtrip("foo <blockquote>~Some code</blockquote> bar");
        }

        #[wasm_bindgen_test]
        fn nbsp_chars_are_removed() {
            let html = "\
                <p>\u{A0}</p>\
                <pre><code>\u{A0}\n\u{A0}</code></pre>\
                <p>\u{A0}</p>";
            let dom = HtmlParser::default().parse::<Utf16String>(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                ├>p
                ├>codeblock
                │ ├>p
                │ └>p
                └>p
                "#}
            );
        }

        #[wasm_bindgen_test]
        fn nbsp_text_is_removed() {
            let html = "\
                <p>&nbsp;</p>\
                <pre><code>&nbsp;\n&nbsp;</code></pre>\
                <p>&nbsp;</p>";
            let dom = HtmlParser::default().parse::<Utf16String>(html).unwrap();
            let tree = dom.to_tree().to_string();
            assert_eq!(
                tree,
                indoc! {
                r#"
                
                ├>p
                ├>codeblock
                │ ├>p
                │ └>p
                └>p
                "#}
            );
        }
    }
}
