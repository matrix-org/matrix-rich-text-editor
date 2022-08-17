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

use crate::dom::dom_handle::DomHandle;
use crate::dom::html_formatter::HtmlFormatter;
use crate::dom::nodes::container_node::ContainerNode;
use crate::dom::nodes::formatting_node::FormattingNode;
use crate::dom::nodes::hyperlink_node::HyperlinkNode;
use crate::dom::nodes::text_node::TextNode;
use crate::dom::to_html::ToHtml;

#[derive(Clone, Debug, PartialEq)]
pub enum DomNode<C> {
    Container(ContainerNode<C>),   // E.g. html, div
    Formatting(FormattingNode<C>), // E.g. b, i
    // TODO Item(ItemNode<C>),             // E.g. a, pills
    Text(TextNode<C>),
    Link(HyperlinkNode<C>),
}

impl<C> DomNode<C> {
    pub fn handle(&self) -> DomHandle {
        match self {
            DomNode::Container(n) => n.handle(),
            DomNode::Formatting(n) => n.handle(),
            DomNode::Text(n) => n.handle(),
            DomNode::Link(n) => n.handle(),
        }
    }

    pub fn set_handle(&mut self, handle: DomHandle) {
        match self {
            DomNode::Container(n) => n.set_handle(handle),
            DomNode::Formatting(n) => n.set_handle(handle),
            DomNode::Text(n) => n.set_handle(handle),
            DomNode::Link(n) => n.set_handle(handle),
        }
    }
}
impl ToHtml<u16> for DomNode<u16> {
    fn fmt_html(&self, f: &mut HtmlFormatter<u16>) {
        match self {
            DomNode::Container(s) => s.fmt_html(f),
            DomNode::Formatting(s) => s.fmt_html(f),
            // TODO DomNode::Item(s) => s.fmt_html(f),
            DomNode::Text(s) => s.fmt_html(f),
            DomNode::Link(s) => s.fmt_html(f),
        }
    }
}
