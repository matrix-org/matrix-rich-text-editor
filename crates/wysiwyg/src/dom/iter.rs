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

use super::{nodes::TextNode, Dom};

impl<S> Dom<S>
where
    S: UnicodeString,
{
    /// Return an iterator over all nodes of this DOM, in depth-first order
    pub fn iter(&self) -> DomIterator<S> {
        DomIterator::over(self)
    }

    /// Return an iterator over all text nodes of this DOM, in depth-first
    /// order
    pub fn iter_text(&self) -> impl Iterator<Item = &TextNode<S>> {
        self.iter().filter_map(|node| {
            if let DomNode::Text(t) = node {
                Some(t)
            } else {
                None
            }
        })
    }
}

pub struct DomIterator<'a, S>
where
    S: UnicodeString,
{
    started: bool,
    ancestors: Vec<(&'a DomNode<S>, usize)>,
}

impl<'a, S> DomIterator<'a, S>
where
    S: UnicodeString,
{
    fn over(dom: &'a Dom<S>) -> Self {
        Self {
            started: false,
            ancestors: vec![(&dom.document_node(), 0)],
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
            if let Some((DomNode::Container(c), idx)) = parent {
                let siblings = c.children();
                if *idx < siblings.len() {
                    let myself = &siblings[*idx];
                    *idx += 1;
                    if let DomNode::Container(_) = myself {
                        self.ancestors.push((myself, 0));
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
            Some(self.ancestors[0].0)
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
        d|<br />\
        <b>x</b>";

    #[test]
    fn can_walk_all_nodes() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let text_nodes: Vec<String> = dom.iter().map(node_txt).collect();

        assert_eq!(
            text_nodes,
            vec![
                "", "ul", "li", "'b'", "strong", "'c'", "li", "'foo'", "'d'",
                "br", "b", "'x'"
            ]
        );
    }

    #[test]
    fn can_walk_all_text_nodes() {
        let dom = cm(EXAMPLE_HTML).state.dom;
        let text_nodes: Vec<String> = dom
            .iter_text()
            .map(|text| text.data().to_string())
            .collect();

        assert_eq!(text_nodes, vec!["b", "c", "foo", "d", "x"]);
    }

    fn node_txt(node: &DomNode<Utf16String>) -> String {
        match node {
            DomNode::Container(c) => c.name().to_string(),
            DomNode::Text(t) => format!("'{}'", t.data()),
            DomNode::LineBreak(_) => String::from("br"),
        }
    }
}
