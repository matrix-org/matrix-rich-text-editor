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

trait Element<'a> {
    fn name(&'a self) -> &'a str;
    fn children(&'a self) -> &'a Vec<DomNode>;
}

fn fmt_element<'a>(
    element: &'a impl Element<'a>,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    let name = element.name();
    if !name.is_empty() {
        f.write_fmt(format_args!("<{}>", name))?;
    }
    for child in element.children() {
        child.fmt(f)?;
    }
    if !name.is_empty() {
        f.write_fmt(format_args!("</{}>", name))?;
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dom {
    document: ContainerNode,
}

impl Dom {
    pub fn new(top_level_items: Vec<DomNode>) -> Self {
        Self {
            document: ContainerNode::new("", top_level_items),
        }
    }

    pub fn children(&self) -> &Vec<DomNode> {
        self.document.children()
    }

    pub fn children_mut(&mut self) -> &mut Vec<DomNode> {
        &mut self.document.children
    }

    pub fn append(&mut self, child: DomNode) {
        self.document.append(child)
    }
}

impl Display for Dom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.document.fmt(f)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContainerNode {
    name: String,
    children: Vec<DomNode>,
}

impl ContainerNode {
    pub fn new(name: &str, children: Vec<DomNode>) -> Self {
        Self {
            name: String::from(name),
            children,
        }
    }

    pub fn append(&mut self, child: DomNode) {
        self.children.push(child);
    }
}

impl<'a> Element<'a> for ContainerNode {
    fn name(&'a self) -> &'a str {
        &self.name
    }

    fn children(&'a self) -> &'a Vec<DomNode> {
        &self.children
    }
}

impl Display for ContainerNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_element(self, f)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FormattingNode {
    name: String,
    children: Vec<DomNode>,
}

impl FormattingNode {
    pub fn new(name: &str, children: Vec<DomNode>) -> Self {
        Self {
            name: String::from(name),
            children,
        }
    }
}

impl<'a> Element<'a> for FormattingNode {
    fn name(&'a self) -> &'a str {
        &self.name
    }

    fn children(&'a self) -> &'a Vec<DomNode> {
        &self.children
    }
}

impl Display for FormattingNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_element(self, f)
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

#[derive(Clone, Debug, PartialEq)]
pub struct TextNode {
    data: String,
}

impl TextNode {
    pub fn from(data: &str) -> Self {
        Self {
            data: String::from(data),
        }
    }

    pub fn data(&self) -> &str {
        &self.data
    }

    pub fn set_data(&mut self, data: String) {
        self.data = data;
    }
}

impl Display for TextNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.data)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DomNode {
    // TODO Container(ContainerNode),   // E.g. html, div
    Formatting(FormattingNode), // E.g. b, i
    // TODO Item(ItemNode),             // E.g. a, pills
    Text(TextNode),
}

impl Display for DomNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // TODO DomNode::Container(s) => s.fmt(f),
            DomNode::Formatting(s) => s.fmt(f),
            // TODO DomNode::Item(s) => s.fmt(f),
            DomNode::Text(s) => s.fmt(f),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn clone_children<'a>(
        children: impl IntoIterator<Item = &'a DomNode>,
    ) -> Vec<DomNode> {
        children.into_iter().cloned().collect()
    }

    fn dom<'a>(children: impl IntoIterator<Item = &'a DomNode>) -> Dom {
        Dom::new(clone_children(children))
    }

    fn b<'a>(children: impl IntoIterator<Item = &'a DomNode>) -> DomNode {
        DomNode::Formatting(FormattingNode::new("b", clone_children(children)))
    }

    fn i<'a>(children: impl IntoIterator<Item = &'a DomNode>) -> DomNode {
        DomNode::Formatting(FormattingNode::new("i", clone_children(children)))
    }

    fn tx(data: &str) -> DomNode {
        DomNode::Text(TextNode::from(data))
    }

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
}
