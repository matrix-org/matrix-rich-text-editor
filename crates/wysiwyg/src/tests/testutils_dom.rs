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

#![cfg(test)]

use crate::dom::nodes::{DomNode, TextNode};
use crate::dom::Dom;

pub fn dom<'a, C>(children: impl IntoIterator<Item = &'a DomNode<C>>) -> Dom<C>
where
    C: 'static + Clone,
{
    Dom::new(clone_children(children))
}

pub fn a<'a>(
    children: impl IntoIterator<Item = &'a DomNode<u16>>,
) -> DomNode<u16> {
    DomNode::new_link(utf16("https://element.io"), clone_children(children))
}

pub fn b<'a>(
    children: impl IntoIterator<Item = &'a DomNode<u16>>,
) -> DomNode<u16> {
    DomNode::new_formatting(utf16("b"), clone_children(children))
}

pub fn i<'a>(
    children: impl IntoIterator<Item = &'a DomNode<u16>>,
) -> DomNode<u16> {
    DomNode::new_formatting(utf16("i"), clone_children(children))
}

pub fn i_c<'a>(
    children: impl IntoIterator<Item = &'a DomNode<u16>>,
) -> DomNode<u16> {
    DomNode::new_formatting(utf16("code"), clone_children(children))
}

fn clone_children<'a, C>(
    children: impl IntoIterator<Item = &'a DomNode<C>>,
) -> Vec<DomNode<C>>
where
    C: 'static + Clone,
{
    children.into_iter().cloned().collect()
}

pub fn tn(data: &str) -> DomNode<u16> {
    DomNode::Text(TextNode::from(utf16(data)))
}

fn utf16(input: &str) -> Vec<u16> {
    input.encode_utf16().collect()
}
