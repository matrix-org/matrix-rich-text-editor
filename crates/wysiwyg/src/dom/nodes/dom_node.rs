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
use crate::dom::nodes::{ContainerNode, LineBreakNode, TextNode};
use crate::dom::to_html::ToHtml;
use crate::dom::to_markdown::{MarkdownError, MarkdownOptions, ToMarkdown};
use crate::dom::to_raw_text::ToRawText;
use crate::dom::to_tree::ToTree;
use crate::dom::unicode_string::UnicodeStrExt;
use crate::dom::UnicodeString;
use crate::{InlineFormatType, ListType};

#[derive(Clone, Debug, PartialEq)]
pub enum DomNode<S>
where
    S: UnicodeString,
{
    Container(ContainerNode<S>), // E.g. html, div
    Text(TextNode<S>),
    LineBreak(LineBreakNode<S>),
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
        DomNode::LineBreak(LineBreakNode::new())
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

    pub fn new_list_item(
        item_name: S,
        children: Vec<DomNode<S>>,
    ) -> DomNode<S> {
        DomNode::Container(ContainerNode::new_list_item(item_name, children))
    }

    pub fn handle(&self) -> DomHandle {
        match self {
            DomNode::Container(n) => n.handle(),
            DomNode::LineBreak(n) => n.handle(),
            DomNode::Text(n) => n.handle(),
        }
    }

    pub fn set_handle(&mut self, handle: DomHandle) {
        match self {
            DomNode::Container(n) => n.set_handle(handle),
            DomNode::LineBreak(n) => n.set_handle(handle),
            DomNode::Text(n) => n.set_handle(handle),
        }
    }

    pub fn text_len(&self) -> usize {
        match self {
            DomNode::Text(n) => n.data().len(),
            DomNode::LineBreak(n) => n.text_len(),
            DomNode::Container(n) => n.text_len(),
        }
    }

    pub fn new_link(url: S, children: Vec<DomNode<S>>) -> DomNode<S> {
        DomNode::Container(ContainerNode::new_link(url, children))
    }

    pub fn is_container_node(&self) -> bool {
        matches!(self, DomNode::Container(_))
    }

    pub fn is_text_node(&self) -> bool {
        matches!(self, DomNode::Text(_))
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

    pub(crate) fn is_placeholder_text_node(&self) -> bool {
        matches!(self, DomNode::Text(n) if n.data().len() == 1 && n.data() == "\u{200b}")
    }

    pub(crate) fn has_only_placeholder_text_child(&self) -> bool {
        match self {
            DomNode::Container(n) => {
                n.children().len() == 1
                    && n.children().first().unwrap().is_placeholder_text_node()
            }
            _ => false,
        }
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
}

impl<S> ToHtml<S> for DomNode<S>
where
    S: UnicodeString,
{
    fn fmt_html(
        &self,
        buf: &mut S,
        selection_writer: Option<&mut SelectionWriter>,
        is_last_node_in_parent: bool,
    ) {
        match self {
            DomNode::Container(s) => {
                s.fmt_html(buf, selection_writer, is_last_node_in_parent)
            }
            DomNode::LineBreak(s) => {
                s.fmt_html(buf, selection_writer, is_last_node_in_parent)
            }
            DomNode::Text(s) => {
                s.fmt_html(buf, selection_writer, is_last_node_in_parent)
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
        }
    }
}
