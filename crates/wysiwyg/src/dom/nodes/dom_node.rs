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
use crate::dom::nodes::text_node::TextNode;
use crate::dom::to_html::ToHtml;
use crate::dom::UnicodeString;

#[derive(Clone, Debug, PartialEq)]
pub enum DomNode<S>
where
    S: UnicodeString,
{
    Container(ContainerNode<S>), // E.g. html, div
    Text(TextNode<S>),
}

impl<S> DomNode<S>
where
    S: UnicodeString,
{
    pub fn new_formatting(format: S, children: Vec<DomNode<S>>) -> DomNode<S> {
        DomNode::Container(ContainerNode::new_formatting(format, children))
    }

    pub fn new_list(list_type: S, children: Vec<DomNode<S>>) -> DomNode<S> {
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
            DomNode::Text(n) => n.handle(),
        }
    }

    pub fn set_handle(&mut self, handle: DomHandle) {
        match self {
            DomNode::Container(n) => n.set_handle(handle),
            DomNode::Text(n) => n.set_handle(handle),
        }
    }

    pub fn text_len(&self) -> usize {
        match self {
            DomNode::Text(n) => n.data().len(),
            DomNode::Container(n) => n.text_len(),
        }
    }

    pub fn new_link(url: S, children: Vec<DomNode<S>>) -> DomNode<S> {
        DomNode::Container(ContainerNode::new_link(url, children))
    }
}

impl<S> ToHtml<S> for DomNode<S>
where
    S: UnicodeString,
{
    fn fmt_html(&self, f: &mut HtmlFormatter<S>) {
        match self {
            DomNode::Container(s) => s.fmt_html(f),
            // TODO DomNode::Item(s) => s.fmt_html(f),
            DomNode::Text(s) => s.fmt_html(f),
        }
    }
}
