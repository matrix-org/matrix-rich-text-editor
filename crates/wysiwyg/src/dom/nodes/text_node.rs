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
use crate::dom::to_html::ToHtml;

use html_escape;

#[derive(Clone, Debug, PartialEq)]
pub struct TextNode<C> {
    data: Vec<C>,
    handle: DomHandle,
}

impl<C> TextNode<C> {
    /// Create a new TextNode
    ///
    /// NOTE: Its handle() will be unset until you call set_handle() or
    /// append() it to another node.
    pub fn from(data: Vec<C>) -> Self
    where
        C: Clone,
    {
        Self {
            data,
            handle: DomHandle::new_unset(),
        }
    }

    pub fn data(&self) -> &[C] {
        &self.data
    }

    pub fn set_data(&mut self, data: Vec<C>) {
        self.data = data;
    }

    pub fn handle(&self) -> DomHandle {
        self.handle.clone()
    }

    pub fn set_handle(&mut self, handle: DomHandle) {
        self.handle = handle;
    }
}

impl ToHtml<u16> for TextNode<u16> {
    fn fmt_html(&self, f: &mut HtmlFormatter<u16>) {
        let string = String::from_utf16(&self.data).unwrap();
        let mut escaped = String::new();
        html_escape::encode_text_to_string(&string, &mut escaped);
        f.write(&escaped.encode_utf16().collect::<Vec<u16>>());
    }
}
