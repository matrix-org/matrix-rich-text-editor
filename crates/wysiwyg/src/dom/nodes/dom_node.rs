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

use crate::composer_model::example_format::SelectionWriter;
use crate::dom::dom_handle::DomHandle;
use crate::dom::nodes::{
    ContainerNode, ContainerNodeKind, LineBreakNode, TextNode,
};
use crate::dom::to_html::{ToHtml, ToHtmlState};
use crate::dom::to_markdown::{MarkdownError, MarkdownOptions, ToMarkdown};
use crate::dom::to_plain_text::ToPlainText;
use crate::dom::to_raw_text::ToRawText;
use crate::dom::to_tree::ToTree;
use crate::dom::unicode_string::UnicodeStrExt;
use crate::dom::{self, UnicodeString};
use crate::{InlineFormatType, ListType};

use super::MentionNode;

#[derive(Clone, Debug, PartialEq)]
pub enum DomNode<S>
where
    S: UnicodeString,
{
    Container(ContainerNode<S>), // E.g. html, div
    Text(TextNode<S>),
    LineBreak(LineBreakNode<S>),
    Mention(MentionNode<S>),
}

impl<S: dom::unicode_string::UnicodeString> Default for DomNode<S> {
    fn default() -> DomNode<S> {
        Self::Container(ContainerNode::default())
    }
}

impl<S> DomNode<S>
where
    S: UnicodeString,
{
    pub fn new_text(text: S) -> DomNode<S> {
        DomNode::Text(TextNode::from(text))
    }

    pub fn new_empty_text() -> DomNode<S> {
        DomNode::Text(TextNode::from(S::default()))
    }

    pub fn new_line_break() -> DomNode<S> {
        DomNode::LineBreak(LineBreakNode::default())
    }

    pub fn new_formatting(
        format: InlineFormatType,
        children: Vec<DomNode<S>>,
    ) -> DomNode<S> {
        DomNode::Container(ContainerNode::new_formatting(format, children))
    }

    pub fn new_formatting_from_tag(
        format: S,
        children: Vec<DomNode<S>>,
    ) -> DomNode<S> {
        DomNode::Container(
            ContainerNode::new_formatting_from_tag(format.clone(), children)
                .unwrap_or_else(|| panic!("Unknown format tag {format}")),
        )
    }

    pub fn new_list(
        list_type: ListType,
        children: Vec<DomNode<S>>,
    ) -> DomNode<S> {
        DomNode::Container(ContainerNode::new_list(list_type, children))
    }

    pub fn new_list_item(children: Vec<DomNode<S>>) -> DomNode<S> {
        DomNode::Container(ContainerNode::new_list_item(children))
    }

    pub fn new_code_block(children: Vec<DomNode<S>>) -> DomNode<S> {
        let children = Self::wrap_children_in_paragraphs_if_needed(children);
        DomNode::Container(ContainerNode::new_code_block(children))
    }

    pub fn new_quote(children: Vec<DomNode<S>>) -> DomNode<S> {
        let children = Self::wrap_children_in_paragraphs_if_needed(children);
        DomNode::Container(ContainerNode::new_quote(children))
    }

    pub fn new_paragraph(children: Vec<DomNode<S>>) -> DomNode<S> {
        DomNode::Container(ContainerNode::new_paragraph(children))
    }

    pub fn handle(&self) -> DomHandle {
        match self {
            DomNode::Container(n) => n.handle(),
            DomNode::LineBreak(n) => n.handle(),
            DomNode::Text(n) => n.handle(),
            DomNode::Mention(n) => n.handle(),
        }
    }

    pub fn set_handle(&mut self, handle: DomHandle) {
        match self {
            DomNode::Container(n) => n.set_handle(handle),
            DomNode::LineBreak(n) => n.set_handle(handle),
            DomNode::Text(n) => n.set_handle(handle),
            DomNode::Mention(n) => n.set_handle(handle),
        }
    }

    pub fn text_len(&self) -> usize {
        match self {
            DomNode::Text(n) => n.data().len(),
            DomNode::LineBreak(n) => n.text_len(),
            DomNode::Container(n) => n.text_len(),
            DomNode::Mention(n) => n.text_len(),
        }
    }

    pub fn new_link(
        url: S,
        children: Vec<DomNode<S>>,
        attributes: Vec<(S, S)>,
    ) -> DomNode<S> {
        DomNode::Container(ContainerNode::new_link(url, children, attributes))
    }

    pub fn new_mention(
        url: S,
        display_text: S,
        attributes: Vec<(S, S)>,
    ) -> DomNode<S> {
        DomNode::Mention(MentionNode::new(url, display_text, attributes))
    }

    pub fn is_container_node(&self) -> bool {
        matches!(self, DomNode::Container(_))
    }

    pub fn is_text_node(&self) -> bool {
        matches!(self, DomNode::Text(_))
    }

    pub fn is_mention_node(&self) -> bool {
        matches!(self, DomNode::Mention(_))
    }

    /// Returns `true` if the dom node is [`LineBreak`].
    ///
    /// [`LineBreak`]: DomNode::LineBreak
    #[must_use]
    pub fn is_line_break(&self) -> bool {
        matches!(self, Self::LineBreak(..))
    }

    /// Returns `true` if thie dom node is not a container i.e. a text node or
    /// a text-like node like a line break.
    pub fn is_leaf(&self) -> bool {
        self.is_text_node() || self.is_line_break() || self.is_mention_node()
    }

    pub fn is_structure_node(&self) -> bool {
        matches!(self, DomNode::Container(n) if n.is_structure_node())
    }

    pub fn is_formatting_node(&self) -> bool {
        matches!(self, DomNode::Container(n) if n.is_formatting_node())
    }

    pub fn is_formatting_node_of_type(
        &self,
        format_type: &InlineFormatType,
    ) -> bool {
        matches!(self, DomNode::Container(n) if n.is_formatting_node_of_type(format_type))
    }

    pub(crate) fn is_block_node(&self) -> bool {
        matches!(self, Self::Container(container) if container.is_block_node())
    }

    pub(crate) fn is_list_item(&self) -> bool {
        matches!(self, Self::Container(container) if container.is_list_item())
    }

    #[allow(dead_code)]
    pub(crate) fn is_list(&self) -> bool {
        matches!(self, Self::Container(container) if container.is_list())
    }

    pub(crate) fn as_text(&self) -> Option<&TextNode<S>> {
        if let Self::Text(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub(crate) fn as_container(&self) -> Option<&ContainerNode<S>> {
        if let Self::Container(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub(crate) fn into_container(self) -> Option<ContainerNode<S>> {
        if let Self::Container(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub(crate) fn as_container_mut(&mut self) -> Option<&mut ContainerNode<S>> {
        if let Self::Container(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn kind(&self) -> DomNodeKind {
        match self {
            DomNode::Text(_) => DomNodeKind::Text,
            DomNode::LineBreak(_) => DomNodeKind::LineBreak,
            DomNode::Container(n) => DomNodeKind::from_container_kind(n.kind()),
            DomNode::Mention(_) => DomNodeKind::Mention,
        }
    }

    /// Returns whether this node, or it's first text-like
    /// child is a line break.
    pub fn has_leading_line_break(&self) -> bool {
        match self {
            DomNode::Container(c) => c.has_leading_line_break(),
            DomNode::Text(_) => false,
            DomNode::LineBreak(_) => true,
            DomNode::Mention(_) => false,
        }
    }

    /// Returns true if given node can be pushed into self without any specific change.
    pub(crate) fn can_push(&self, other_node: &DomNode<S>) -> bool {
        match (self, other_node) {
            (DomNode::Container(c1), DomNode::Container(c2)) => {
                c1.kind() == c2.kind()
                    && !c1.is_list_item()
                    && !matches!(c1.kind(), ContainerNodeKind::Paragraph)
            }
            (DomNode::Text(_), DomNode::Text(_)) => true,
            _ => false,
        }
    }

    /// Push content of the given node into self. Panics if pushing is not possible.
    pub(crate) fn push(&mut self, other_node: &mut DomNode<S>) {
        if !self.can_push(other_node) {
            panic!("Trying to push incompatible nodes")
        }

        match self {
            DomNode::Container(c) => {
                c.push(other_node.as_container_mut().unwrap())
            }
            DomNode::Text(t) => t.push(other_node.as_text().unwrap()),
            _ => unreachable!(),
        }
    }

    /// Slice this node after given position.
    /// Returns a new node of the same kind with the
    /// removed content, with both nodes keeping
    /// expected hierarchy.
    pub fn slice_after(&mut self, position: usize) -> Self {
        match self {
            DomNode::Container(c) => {
                DomNode::Container(c.slice_after(position))
            }
            DomNode::Text(t) => DomNode::Text(t.slice_after(position)),
            DomNode::LineBreak(_) => panic!("Can't slice a linebreak"),
            DomNode::Mention(_) => panic!("Can't slice a mention"),
        }
    }

    /// Slice this node before given position.
    /// Returns a new node of the same kind with the
    /// removed content, with both nodes keeping
    /// expected hierarchy.
    pub fn slice_before(&mut self, position: usize) -> Self {
        match self {
            DomNode::Container(c) => {
                DomNode::Container(c.slice_before(position))
            }
            DomNode::Text(t) => DomNode::Text(t.slice_before(position)),
            DomNode::LineBreak(_) => panic!("Can't slice a linebreak"),
            DomNode::Mention(_) => panic!("Can't slice a linebreak"),
        }
    }

    /// Find the node based on its handle.
    /// Panics if the handle is unset or invalid
    pub fn lookup_node(&self, node_handle: &DomHandle) -> &DomNode<S> {
        fn nth_child<S>(element: &ContainerNode<S>, idx: usize) -> &DomNode<S>
        where
            S: UnicodeString,
        {
            element.children().get(idx).unwrap_or_else(|| {
                panic!(
                    "Handle is invalid: it refers to a child index ({}) which \
                is too large for the number of children in this node ({:?}).",
                    idx, element
                )
            })
        }

        let mut node = self;
        if !node_handle.is_set() {
            panic!(
                "Attempting to lookup a node using an unset DomHandle ({:?})",
                node_handle.raw()
            );
        }
        for idx in node_handle.raw() {
            node = match node {
                DomNode::Container(n) => nth_child(n, *idx),
                DomNode::LineBreak(_) => panic!(
                    "Handle is invalid: refers to the child of a line break, \
                    but line breaks cannot have children."
                ),
                DomNode::Text(_) => panic!(
                    "Handle {:?} is invalid: refers to the child of a text node, \
                    but text nodes cannot have children.", node_handle
                ),
                DomNode::Mention(_) => panic!(
                    "Handle {:?} is invalid: refers to the child of a mention node, \
                    but text nodes cannot have children.", node_handle
                ),
            }
        }
        node
    }

    pub fn is_empty(&self) -> bool {
        match self {
            DomNode::Container(container) => container.is_empty(),
            DomNode::Text(text_node) => text_node.data().is_empty(),
            _ => false,
        }
    }

    /// Returns true if there is no text in this DomNode.
    pub fn has_no_text(&self) -> bool {
        match self {
            DomNode::Container(c) => c.has_no_text(),
            DomNode::Text(t) => t.data().is_empty(),
            _ => false,
        }
    }

    fn wrap_children_in_paragraphs_if_needed(
        children: Vec<DomNode<S>>,
    ) -> Vec<DomNode<S>> {
        let all_block_nodes = if children.is_empty() {
            false
        } else {
            children.iter().all(|c| c.is_block_node())
        };
        if !all_block_nodes {
            vec![DomNode::new_paragraph(children)]
        } else {
            children
        }
    }
}

impl<S> ToHtml<S> for DomNode<S>
where
    S: UnicodeString,
{
    fn fmt_html(
        &self,
        buf: &mut S,
        selection_writer: Option<&mut SelectionWriter>,
        state: ToHtmlState,
        as_message: bool,
    ) {
        match self {
            DomNode::Container(s) => {
                s.fmt_html(buf, selection_writer, state, as_message)
            }
            DomNode::LineBreak(s) => {
                s.fmt_html(buf, selection_writer, state, as_message)
            }
            DomNode::Text(s) => {
                s.fmt_html(buf, selection_writer, state, as_message)
            }
            DomNode::Mention(s) => {
                s.fmt_html(buf, selection_writer, state, as_message)
            }
        }
    }
}

impl<S> ToRawText<S> for DomNode<S>
where
    S: UnicodeString,
{
    fn to_raw_text(&self) -> S {
        match self {
            DomNode::Container(n) => n.to_raw_text(),
            DomNode::LineBreak(n) => n.to_raw_text(),
            DomNode::Text(n) => n.to_raw_text(),
            DomNode::Mention(n) => n.to_raw_text(),
        }
    }
}

impl<S> ToPlainText<S> for DomNode<S>
where
    S: UnicodeString,
{
    fn to_plain_text(&self) -> S {
        match self {
            DomNode::Container(n) => n.to_plain_text(),
            DomNode::LineBreak(n) => n.to_plain_text(),
            DomNode::Text(n) => n.to_plain_text(),
            DomNode::Mention(n) => n.to_plain_text(),
        }
    }
}

impl<S> ToTree<S> for DomNode<S>
where
    S: UnicodeString,
{
    fn to_tree_display(&self, continuous_positions: Vec<usize>) -> S {
        match self {
            DomNode::Container(n) => n.to_tree_display(continuous_positions),
            DomNode::LineBreak(n) => n.to_tree_display(continuous_positions),
            DomNode::Text(n) => n.to_tree_display(continuous_positions),
            DomNode::Mention(n) => n.to_tree_display(continuous_positions),
        }
    }
}

impl<S> ToMarkdown<S> for DomNode<S>
where
    S: UnicodeString,
{
    fn fmt_markdown(
        &self,
        buffer: &mut S,
        options: &MarkdownOptions,
    ) -> Result<(), MarkdownError<S>> {
        match self {
            DomNode::Container(container) => {
                container.fmt_markdown(buffer, options)
            }
            DomNode::Text(text) => text.fmt_markdown(buffer, options),
            DomNode::LineBreak(node) => node.fmt_markdown(buffer, options),
            DomNode::Mention(node) => node.fmt_markdown(buffer, options),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DomNodeKind {
    Generic, // Should only be used for root node so far
    Text,
    LineBreak,
    Mention,
    Formatting(InlineFormatType),
    Link,
    ListItem,
    List,
    CodeBlock,
    Quote,
    Paragraph,
}

impl DomNodeKind {
    pub fn from_container_kind<S: UnicodeString>(
        kind: &ContainerNodeKind<S>,
    ) -> Self {
        match kind {
            ContainerNodeKind::Formatting(f) => {
                DomNodeKind::Formatting(f.clone())
            }
            ContainerNodeKind::Link(_) => DomNodeKind::Link,
            ContainerNodeKind::List(_) => DomNodeKind::List,
            ContainerNodeKind::ListItem => DomNodeKind::ListItem,
            ContainerNodeKind::Generic => DomNodeKind::Generic,
            ContainerNodeKind::CodeBlock => DomNodeKind::CodeBlock,
            ContainerNodeKind::Quote => DomNodeKind::Quote,
            ContainerNodeKind::Paragraph => DomNodeKind::Paragraph,
        }
    }

    pub fn is_structure_kind(&self) -> bool {
        matches!(self, Self::List | Self::ListItem)
    }

    pub fn is_block_kind(&self) -> bool {
        matches!(
            self,
            Self::Generic
                | Self::List
                | Self::ListItem
                | Self::CodeBlock
                | Self::Quote
                | Self::Paragraph
        )
    }

    pub fn is_leaf_kind(&self) -> bool {
        match self {
            Self::Text | Self::LineBreak | Self::Mention => true,
            Self::Generic
            | Self::Formatting(_)
            | Self::Link
            | Self::ListItem
            | Self::List
            | Self::CodeBlock
            | Self::Quote
            | Self::Paragraph => false,
        }
    }

    pub fn is_code_kind(&self) -> bool {
        matches!(
            self,
            Self::CodeBlock | Self::Formatting(InlineFormatType::InlineCode)
        )
    }
}

#[cfg(test)]
mod test {
    use widestring::Utf16String;

    use crate::{DomHandle, DomNode, InlineFormatType, ToHtml, UnicodeString};

    #[test]
    fn pushing_nodes_of_same_kind() {
        let mut n1 = format_container_with_handle_and_children(
            InlineFormatType::Bold,
            vec![text_node("abc")],
            &[0, 0],
        );
        let mut n2 = format_container_with_handle_and_children(
            InlineFormatType::Bold,
            vec![text_node("def")],
            &[0, 1],
        );

        assert!(n1.can_push(&n2));
        n1.push(&mut n2);

        let expected = format_container_with_handle_and_children(
            InlineFormatType::Bold,
            vec![text_node("abcdef")],
            &[0, 0],
        );
        assert_eq!(n1, expected);
    }

    #[test]
    fn pushing_nodes_of_different_kind_is_not_allowed() {
        let n1 = format_container_with_handle_and_children(
            InlineFormatType::Bold,
            vec![text_node("abc")],
            &[0, 0],
        );
        let n2 = format_container_with_handle_and_children(
            InlineFormatType::Italic,
            vec![text_node("def")],
            &[0, 1],
        );

        assert!(!n1.can_push(&n2));
    }

    #[test]
    fn pushing_list_item_directly_is_not_allowed() {
        let li1 = list_item_with_handle(&[0, 0]);
        let li2 = list_item_with_handle(&[0, 1]);
        assert!(!li1.can_push(&li2));
    }

    #[test]
    #[should_panic]
    fn pushing_nodes_of_different_kind_panics() {
        let mut n1 = format_container_with_handle_and_children(
            InlineFormatType::Bold,
            vec![text_node("abc")],
            &[0, 0],
        );
        let mut n2 = format_container_with_handle_and_children(
            InlineFormatType::Italic,
            vec![text_node("def")],
            &[0, 1],
        );
        n1.push(&mut n2);
    }

    #[test]
    fn slicing_node() {
        let mut node = format_container_with_nested_children();
        let mut before = node.slice_before(4);
        assert_eq!(before.to_html(), "<del><em>abc</em>d</del>");
        assert_eq!(node.to_html(), "<del>ef</del>");
        before.push(&mut node);
        assert_eq!(before.to_html(), "<del><em>abc</em>def</del>");
    }

    #[test]
    fn slicing_node_on_edge_removes_nothing() {
        let mut node = format_container_with_nested_children();
        node.slice_after(6);
        node.slice_before(0);
        assert_eq!(node.to_html(), "<del><em>abc</em>def</del>")
    }

    #[test]
    #[should_panic]
    fn slicing_over_edge_panics() {
        let mut node = format_container_with_nested_children();
        node.slice_after(42);
        assert_eq!(node.to_html(), "<del><em>abc</em>def</del>");
    }

    /// Result HTML is "<del><em>abc</em>def</del>".
    fn format_container_with_nested_children() -> DomNode<Utf16String> {
        let italic = format_container_with_handle_and_children(
            InlineFormatType::Italic,
            vec![text_node("abc")],
            &[0, 0],
        );
        format_container_with_handle_and_children(
            InlineFormatType::StrikeThrough,
            vec![italic, text_node("def")],
            &[0, 0],
        )
    }

    fn format_container_with_handle_and_children<'a>(
        format: InlineFormatType,
        children: Vec<DomNode<Utf16String>>,
        raw_handle: impl IntoIterator<Item = &'a usize>,
    ) -> DomNode<Utf16String> {
        let mut node = DomNode::new_formatting(format, children);
        let handle =
            DomHandle::from_raw(raw_handle.into_iter().cloned().collect());
        node.set_handle(handle);
        node
    }

    fn list_item_with_handle<'a>(
        raw_handle: impl IntoIterator<Item = &'a usize>,
    ) -> DomNode<Utf16String> {
        let mut node = DomNode::new_list_item(vec![]);
        let handle =
            DomHandle::from_raw(raw_handle.into_iter().cloned().collect());
        node.set_handle(handle);
        node
    }

    fn text_node<S>(content: &str) -> DomNode<S>
    where
        S: UnicodeString,
    {
        DomNode::new_text(content.into())
    }
}
