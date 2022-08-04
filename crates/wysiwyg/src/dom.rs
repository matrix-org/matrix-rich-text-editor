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

fn utf8(input: &[u16]) -> String {
    String::from_utf16(input).expect("Invalid UTF-16!")
}

trait Element<'a, C> {
    fn name(&'a self) -> &'a Vec<C>;
    fn children(&'a self) -> &'a Vec<DomNode<C>>;
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

fn fmt_element_u16<'a>(
    element: &'a impl Element<'a, u16>,
    f: &mut HtmlFormatter<u16>,
) {
    fmt_element(element, '<' as u16, '>' as u16, '/' as u16, f);
}

pub struct HtmlFormatter<C> {
    chars: Vec<C>,
}

impl<C> HtmlFormatter<C>
where
    C: Clone,
{
    fn new() -> Self {
        Self { chars: Vec::new() }
    }

    fn write_char(&mut self, c: &C) {
        self.chars.push(c.clone());
    }

    fn write(&mut self, slice: &[C]) {
        self.chars.extend_from_slice(slice);
    }

    fn finish(self) -> Vec<C> {
        self.chars
    }
}

pub trait ToHtml<C>
where
    C: Clone,
{
    fn fmt_html(&self, f: &mut HtmlFormatter<C>);

    fn to_html(&self) -> Vec<C> {
        let mut f = HtmlFormatter::new();
        self.fmt_html(&mut f);
        f.finish()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dom<C> {
    document: ContainerNode<C>,
}

impl<C> Dom<C> {
    pub fn new(top_level_items: Vec<DomNode<C>>) -> Self {
        Self {
            document: ContainerNode::new(Vec::new(), top_level_items),
        }
    }

    pub fn children(&self) -> &Vec<DomNode<C>> {
        self.document.children()
    }

    pub fn children_mut(&mut self) -> &mut Vec<DomNode<C>> {
        &mut self.document.children
    }

    pub fn append(&mut self, child: DomNode<C>) {
        self.document.append(child)
    }
}

impl<C> ToHtml<C> for Dom<C>
where
    C: Clone,
    ContainerNode<C>: ToHtml<C>,
{
    fn fmt_html(&self, f: &mut HtmlFormatter<C>) {
        self.document.fmt_html(f)
    }
}

impl Display for Dom<u16> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&utf8(&self.to_html()))?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContainerNode<C> {
    name: Vec<C>,
    children: Vec<DomNode<C>>,
}

impl<C> ContainerNode<C> {
    pub fn new(name: Vec<C>, children: Vec<DomNode<C>>) -> Self {
        Self { name, children }
    }

    pub fn append(&mut self, child: DomNode<C>) {
        self.children.push(child);
    }
}

impl<'a, C> Element<'a, C> for ContainerNode<C> {
    fn name(&'a self) -> &'a Vec<C> {
        &self.name
    }

    fn children(&'a self) -> &'a Vec<DomNode<C>> {
        &self.children
    }
}

impl ToHtml<u16> for ContainerNode<u16> {
    fn fmt_html(&self, f: &mut HtmlFormatter<u16>) {
        fmt_element_u16(self, f)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FormattingNode<C> {
    name: Vec<C>,
    children: Vec<DomNode<C>>,
}

impl<C> FormattingNode<C> {
    pub fn new(name: Vec<C>, children: Vec<DomNode<C>>) -> Self {
        Self { name, children }
    }
}

impl<'a, C> Element<'a, C> for FormattingNode<C> {
    fn name(&'a self) -> &'a Vec<C> {
        &self.name
    }

    fn children(&'a self) -> &'a Vec<DomNode<C>> {
        &self.children
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

#[derive(Clone, Debug, PartialEq)]
pub struct TextNode<C> {
    data: Vec<C>,
}

impl<C> TextNode<C> {
    pub fn from(data: Vec<C>) -> Self
    where
        C: Clone,
    {
        Self { data }
    }

    pub fn data(&self) -> &[C] {
        &self.data
    }

    pub fn set_data(&mut self, data: Vec<C>) {
        self.data = data;
    }
}

impl ToHtml<u16> for TextNode<u16> {
    fn fmt_html(&self, f: &mut HtmlFormatter<u16>) {
        f.write(&self.data)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DomNode<C> {
    // TODO Container(ContainerNode<C>),   // E.g. html, div
    Formatting(FormattingNode<C>), // E.g. b, i
    // TODO Item(ItemNode<C>),             // E.g. a, pills
    Text(TextNode<C>),
}

impl ToHtml<u16> for DomNode<u16> {
    fn fmt_html(&self, f: &mut HtmlFormatter<u16>) {
        match self {
            // TODO DomNode::Container(s) => s.fmt_html(f),
            DomNode::Formatting(s) => s.fmt_html(f),
            // TODO DomNode::Item(s) => s.fmt_html(f),
            DomNode::Text(s) => s.fmt_html(f),
        }
    }
}

#[cfg(test)]
mod test {
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
