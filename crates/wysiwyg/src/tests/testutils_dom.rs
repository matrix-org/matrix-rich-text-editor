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

use widestring::Utf16String;

use crate::dom::nodes::DomNode;
use crate::dom::Dom;

use crate::tests::testutils_conversion::utf16;

pub fn dom<'a>(
    children: impl IntoIterator<Item = &'a DomNode<Utf16String>>,
) -> Dom<Utf16String> {
    Dom::new(clone_children(children))
}

pub fn a<'a>(
    children: impl IntoIterator<Item = &'a DomNode<Utf16String>>,
) -> DomNode<Utf16String> {
    DomNode::new_link(utf16("https://element.io"), clone_children(children))
}

pub fn b<'a>(
    children: impl IntoIterator<Item = &'a DomNode<Utf16String>>,
) -> DomNode<Utf16String> {
    DomNode::new_formatting_from_tag(utf16("b"), clone_children(children))
}

pub fn br<'a>() -> DomNode<Utf16String> {
    DomNode::new_line_break()
}

pub fn i<'a>(
    children: impl IntoIterator<Item = &'a DomNode<Utf16String>>,
) -> DomNode<Utf16String> {
    DomNode::new_formatting_from_tag(utf16("i"), clone_children(children))
}

pub fn i_c<'a>(
    children: impl IntoIterator<Item = &'a DomNode<Utf16String>>,
) -> DomNode<Utf16String> {
    DomNode::new_formatting_from_tag(utf16("code"), clone_children(children))
}

fn clone_children<'a>(
    children: impl IntoIterator<Item = &'a DomNode<Utf16String>>,
) -> Vec<DomNode<Utf16String>> {
    children.into_iter().cloned().collect()
}

pub fn tn(data: &str) -> DomNode<Utf16String> {
    DomNode::new_text(utf16(data))
}
