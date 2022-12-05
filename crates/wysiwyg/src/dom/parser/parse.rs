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

use crate::dom::{Dom, DomCreationError, UnicodeString};

pub fn parse<S>(html: &str) -> Result<Dom<S>, DomCreationError<S>>
where
    S: UnicodeString,
{
    cfg_if::cfg_if! {
        if #[cfg(feature = "sys")] {
            sys::parse(html)
        } else if #[cfg(all(feature = "js", target_arch = "wasm32"))] {
            js::parse(html)
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
    use crate::dom::nodes::{ContainerNode, DomNode};
    use crate::ListType;

    pub(super) fn parse<S>(html: &str) -> Result<Dom<S>, DomCreationError<S>>
    where
        S: UnicodeString,
    {
        PaDomCreator::parse(html)
            .map(padom_to_dom)
            .map_err(padom_creation_error_to_dom_creation_error)
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
    fn padom_to_dom<S>(padom: PaDom) -> Dom<S>
    where
        S: UnicodeString,
    {
        /// Recurse into panode's children and convert them too
        fn convert_children<S>(
            padom: &PaDom,
            child: &PaNodeContainer,
            new_node: Option<&mut DomNode<S>>,
        ) where
            S: UnicodeString,
        {
            if let DomNode::Container(new_node) = new_node.unwrap() {
                convert(padom, child, new_node);
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
            DomNode::Container(ContainerNode::new_link(
                child.get_attr("href").unwrap_or("").into(),
                Vec::new(),
            ))
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

        /// Copy all panode's information into node (now we know it's a container).
        fn convert_container<S>(
            padom: &PaDom,
            child: &PaNodeContainer,
            node: &mut ContainerNode<S>,
        ) where
            S: UnicodeString,
        {
            let tag = child.name.local.as_ref();
            match tag {
                "b" | "code" | "del" | "em" | "i" | "strong" | "u" => {
                    node.append_child(new_formatting(tag));
                    convert_children(padom, child, node.last_child_mut());
                }
                "br" => {
                    node.append_child(new_line_break());
                }
                "ol" | "ul" => {
                    node.append_child(new_list(tag));
                    convert_children(padom, child, node.last_child_mut());
                }
                "li" => {
                    node.append_child(new_list_item());
                    convert_children(padom, child, node.last_child_mut());
                }
                "a" => {
                    node.append_child(new_link(child));
                    convert_children(padom, child, node.last_child_mut());
                }
                "html" => {
                    // Skip the html tag - add its children to the
                    // current node directly.
                    convert(padom, child, node);
                }
                _ => {
                    // Ignore tags we don't recognise
                    // We should log - see internal task PSU-741
                }
            };
        }

        /// Copy all panode's information into node.
        fn convert<S>(
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
                        convert_container(padom, child, node);
                    }
                    PaDomNode::Document(_) => {
                        panic!("Found a document inside a document!")
                    }
                    PaDomNode::Text(text) => {
                        node.append_child(DomNode::new_text(
                            text.content.as_str().into(),
                        ));
                    }
                }
            }
        }

        let mut ret = Dom::new(Vec::new());
        let doc = ret.document_mut();

        if let PaDomNode::Document(padoc) = padom.get_document() {
            convert(&padom, padoc, doc)
        } else {
            panic!("Document was not a document!");
        }

        ret
    }

    fn padom_creation_error_to_dom_creation_error<S>(
        e: PaDomCreationError,
    ) -> DomCreationError<S>
    where
        S: UnicodeString,
    {
        DomCreationError {
            dom: padom_to_dom(e.dom),
            parse_errors: e.parse_errors,
        }
    }

    #[cfg(test)]
    mod test {
        use speculoos::{assert_that, AssertionFailure, Spec};
        use widestring::Utf16String;

        use crate::tests::testutils_composer_model::restore_whitespace;
        use crate::ToHtml;

        use super::parse;

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
    }
}

#[cfg(all(feature = "js", target_arch = "wasm32"))]
mod js {
    use super::*;
    use crate::{
        dom::nodes::{ContainerNode, DomNode},
        InlineFormatType, ListType,
    };
    use std::fmt;
    use wasm_bindgen::JsCast;
    use web_sys::{Document, DomParser, Element, NodeList, SupportedType};

    pub(super) fn parse<S>(html: &str) -> Result<Dom<S>, DomCreationError<S>>
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

        webdom_to_dom(document).map_err(to_dom_creation_error)
    }

    fn webdom_to_dom<S>(webdoc: Document) -> Result<Dom<S>, Error>
    where
        S: UnicodeString,
    {
        let body = webdoc.body().ok_or_else(|| Error::NoBody)?;

        fn convert<S>(nodes: NodeList) -> Result<Dom<S>, Error>
        where
            S: UnicodeString,
        {
            let number_of_nodes = nodes.length() as usize;
            let mut dom = Dom::new(Vec::with_capacity(number_of_nodes));
            let dom_document = dom.document_mut();

            convert_container(nodes, dom_document)?;

            Ok(dom)
        }

        fn convert_container<S>(
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

                    "#text" => {
                        dom.append_child(match node.node_value() {
                            Some(value) => {
                                DomNode::new_text(value.as_str().into())
                            }
                            None => DomNode::new_empty_text(),
                        });
                    }

                    "A" => {
                        dom.append_child(DomNode::new_link(
                            node.unchecked_ref::<Element>()
                                .get_attribute("href")
                                .unwrap_or_default()
                                .into(),
                            convert(node.child_nodes())?.take_children(),
                        ));
                    }

                    "OL" => {
                        dom.append_child(DomNode::Container(
                            ContainerNode::new_list(
                                ListType::Ordered,
                                convert(node.child_nodes())?.take_children(),
                            ),
                        ));
                    }

                    "UL" => {
                        dom.append_child(DomNode::Container(
                            ContainerNode::new_list(
                                ListType::Unordered,
                                convert(node.child_nodes())?.take_children(),
                            ),
                        ));
                    }

                    "LI" => {
                        dom.append_child(DomNode::Container(
                            ContainerNode::new_list_item(
                                convert(node.child_nodes())?.take_children(),
                            ),
                        ));
                    }

                    node_name => {
                        let children_nodes =
                            convert(node.child_nodes())?.take_children();

                        dom.append_child(DomNode::Container(
                            ContainerNode::new_formatting(
                                match node_name {
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
                                },
                                children_nodes,
                            ),
                        ));
                    }
                }
            }

            Ok(())
        }

        convert(body.child_nodes())
    }

    fn to_dom_creation_error<S, E>(error: E) -> DomCreationError<S>
    where
        S: UnicodeString,
        E: ToString,
    {
        DomCreationError {
            dom: Dom::new(vec![]),
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
            tests::testutils_composer_model::restore_whitespace, ToHtml,
        };
        use wasm_bindgen_test::*;
        use widestring::Utf16String;

        wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

        fn roundtrip(html: &str) {
            let parse = parse::<Utf16String>(html);

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
        fn ul() {
            roundtrip("foo <ul><li>item1</li><li>item2</li></ul> bar");
        }

        #[wasm_bindgen_test]
        fn ol() {
            roundtrip("foo <ol><li>item1</li><li>item2</li></ol> bar");
        }
    }
}
