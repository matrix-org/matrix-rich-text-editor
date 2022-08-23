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

use crate::dom::nodes::{ContainerNode, DomNode, TextNode};
use crate::dom::parser::PaNodeContainer;
use crate::dom::{Dom, DomCreationError};
use crate::ToHtml;

use super::padom_node::PaDomNode;
use super::{PaDom, PaDomCreationError, PaDomCreator};

pub fn parse(html: &str) -> Result<Dom<u16>, DomCreationError<u16>> {
    PaDomCreator::parse(html)
        .map(padom_to_dom_u16)
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
fn padom_to_dom_u16(padom: PaDom) -> Dom<u16> {
    /// Recurse into panode's children and convert them too
    fn convert_children(
        padom: &PaDom,
        child: &PaNodeContainer,
        new_node: Option<&mut DomNode<u16>>,
    ) {
        if let DomNode::Container(new_node) = new_node.unwrap() {
            convert(padom, child, new_node);
        } else {
            panic!("Container became non-container!");
        }
    }

    /// Create a formatting node
    fn new_formatting(tag: &str) -> DomNode<u16> {
        DomNode::Container(ContainerNode::new_formatting(
            tag.to_html(),
            Vec::new(),
        ))
    }

    /// Create a link node
    fn new_link(child: &PaNodeContainer) -> DomNode<u16> {
        DomNode::Container(ContainerNode::new_link(
            child.get_attr("href").unwrap_or("").to_html(),
            Vec::new(),
        ))
    }

    /// Create a list node
    fn new_list(tag: &str) -> DomNode<u16> {
        DomNode::Container(ContainerNode::new_list(tag.to_html(), Vec::new()))
    }

    /// Create a list item node
    fn new_list_item(tag: &str) -> DomNode<u16> {
        DomNode::Container(ContainerNode::new_list_item(
            tag.to_html(),
            Vec::new(),
        ))
    }

    /// Copy all panode's information into node (now we know it's a container).
    fn convert_container(
        padom: &PaDom,
        child: &PaNodeContainer,
        node: &mut ContainerNode<u16>,
    ) {
        let tag = child.name.local.as_ref();
        match tag {
            "b" | "code" | "del" | "em" | "i" | "strong" | "u" => {
                node.append(new_formatting(tag));
                convert_children(padom, child, node.children_mut().last_mut());
            }
            "ol" | "ul" => {
                node.append(new_list(tag));
                convert_children(padom, child, node.children_mut().last_mut());
            }
            "li" => {
                node.append(new_list_item(tag));
                convert_children(padom, child, node.children_mut().last_mut());
            }
            "a" => {
                node.append(new_link(child));
                convert_children(padom, child, node.children_mut().last_mut());
            }
            "html" => {
                // Skip the html tag - add its children to the
                // current node directly.
                convert(padom, child, node);
            }
            _ => {
                // Ignore tags we don't recognise
                // TODO: log warning (error?)  - we are ignoring?
            }
        };
    }

    /// Copy all panode's information into node.
    fn convert(
        padom: &PaDom,
        panode: &PaNodeContainer,
        node: &mut ContainerNode<u16>,
    ) {
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
                    node.append(DomNode::Text(TextNode::from(
                        text.content.to_html(),
                    )));
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

fn padom_creation_error_to_dom_creation_error(
    e: PaDomCreationError,
) -> DomCreationError<u16> {
    DomCreationError {
        dom: padom_to_dom_u16(e.dom),
        parse_errors: e.parse_errors,
    }
}

#[cfg(test)]
mod test {
    use speculoos::{assert_that, AssertionFailure, Spec};

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
            let dom = parse(subject).unwrap();
            let output = String::from_utf16(&dom.to_html()).unwrap();
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
}
