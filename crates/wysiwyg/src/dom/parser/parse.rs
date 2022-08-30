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
use crate::dom::{Dom, DomCreationError, UnicodeString};

use super::padom_node::PaDomNode;
use super::{PaDom, PaDomCreationError, PaDomCreator};

pub fn parse<S>(html: &str) -> Result<Dom<S>, DomCreationError<S>>
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
            ContainerNode::new_formatting_from_tag(
                S::from_str(tag),
                Vec::new(),
            )
            .expect(&format!("Unknown format tag {}", tag)),
        )
    }

    /// Create a link node
    fn new_link<S>(child: &PaNodeContainer) -> DomNode<S>
    where
        S: UnicodeString,
    {
        DomNode::Container(ContainerNode::new_link(
            S::from_str(child.get_attr("href").unwrap_or("")),
            Vec::new(),
        ))
    }

    /// Create a list node
    fn new_list<S>(tag: &str) -> DomNode<S>
    where
        S: UnicodeString,
    {
        DomNode::Container(ContainerNode::new_list(
            S::from_str(tag),
            Vec::new(),
        ))
    }

    /// Create a list item node
    fn new_list_item<S>(tag: &str) -> DomNode<S>
    where
        S: UnicodeString,
    {
        DomNode::Container(ContainerNode::new_list_item(
            S::from_str(tag),
            Vec::new(),
        ))
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
            "ol" | "ul" => {
                node.append_child(new_list(tag));
                convert_children(padom, child, node.last_child_mut());
            }
            "li" => {
                node.append_child(new_list_item(tag));
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
                    node.append_child(DomNode::Text(TextNode::from(
                        S::from_str(&text.content),
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

    use crate::{dom::UnicodeString, ToHtml};

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
            let output = dom.to_html().to_utf8();
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
