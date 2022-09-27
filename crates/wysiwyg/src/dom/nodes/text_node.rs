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
use crate::dom::unicode_string::{UnicodeStrExt, UnicodeStringExt};
use html_escape;

use crate::dom::dom_handle::DomHandle;
use crate::dom::to_html::ToHtml;
use crate::dom::to_raw_text::ToRawText;
use crate::dom::to_tree::ToTree;
use crate::dom::UnicodeString;

#[derive(Clone, Debug, PartialEq)]
pub struct TextNode<S>
where
    S: UnicodeString,
{
    data: S,
    handle: DomHandle,
}

impl<S> TextNode<S>
where
    S: UnicodeString,
{
    /// Create a new TextNode
    ///
    /// NOTE: Its handle() will be unset until you call set_handle() or
    /// append() it to another node.
    pub fn from(data: S) -> Self {
        Self {
            data,
            handle: DomHandle::new_unset(),
        }
    }

    pub fn data(&self) -> &S::Str {
        &self.data
    }

    pub fn set_data(&mut self, data: S) {
        self.data = data;
    }

    pub fn handle(&self) -> DomHandle {
        self.handle.clone()
    }

    pub fn set_handle(&mut self, handle: DomHandle) {
        self.handle = handle;
    }
}

impl<S> ToHtml<S> for TextNode<S>
where
    S: UnicodeString,
{
    fn fmt_html(
        &self,
        buf: &mut S,
        selection_writer: Option<&mut SelectionWriter>,
        is_last_node_in_parent: bool,
    ) {
        let cur_pos = buf.len();
        let string = self.data.to_string();

        let mut escaped = html_escape::encode_text(&string)
            // Replace all pairs of spaces with non-breaking ones. Transforms
            // `a     b` to `a\u{A0}\u{A0}\u{A0}\u{A0} b`, which will render
            // exactly as five spaces like in the input.
            .replace("  ", "\u{A0}\u{A0}");
        if is_last_node_in_parent
            && escaped.chars().next_back().map_or(false, |c| c == ' ')
        {
            // If this is the last node and it ends in a space, replace that
            // space with a non-breaking one.
            escaped.replace_range(escaped.len() - 1.., "\u{A0}");
        }
        buf.push(escaped.as_str());

        if let Some(selection_writer) = selection_writer {
            selection_writer.write_selection_text_node(buf, cur_pos, self);
        }
    }
}

impl<S> ToRawText<S> for TextNode<S>
where
    S: UnicodeString,
{
    fn to_raw_text(&self) -> S {
        self.data.clone()
    }
}

impl<S> ToTree<S> for TextNode<S>
where
    S: UnicodeString,
{
    fn to_tree_display(&self, continuous_positions: Vec<usize>) -> S {
        let mut description = S::from("\"");
        let text = &self.data.to_string().replace('\u{200b}', "~");
        description.push(text.as_str());
        description.push('"');
        return self.tree_line(
            description,
            self.handle.raw().len(),
            continuous_positions,
        );
    }
}
