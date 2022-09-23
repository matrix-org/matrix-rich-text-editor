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
use html_escape;

use crate::dom::dom_handle::DomHandle;
use crate::dom::html_formatter::HtmlFormatter;
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

    pub fn data(&self) -> &S {
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

    fn handle_several_whitespaces(
        formatter: &mut HtmlFormatter<S>,
        pos: usize,
        is_last_node_in_parent: bool,
    ) {
        let mut ranges_to_replace = Vec::new();
        let mut current_range = usize::MAX..usize::MAX;
        let mut whitespaces: usize = 0;
        let mut needs_to_replace = false;
        let w = S::c_from_char(' ');
        let nbsp = S::c_from_char('\u{A0}');
        for (i, c) in formatter.chars_from(pos..).iter().enumerate() {
            if *c == w || *c == nbsp {
                if *c == w {
                    needs_to_replace = true;
                }
                if current_range.start == usize::MAX {
                    whitespaces = 1;
                    current_range.start = i + pos;
                } else {
                    whitespaces += 1;
                }
            } else {
                if needs_to_replace && whitespaces > 1 {
                    current_range.end = i + pos;
                    ranges_to_replace
                        .push((current_range.clone(), whitespaces));
                }
                needs_to_replace = false;
                current_range = usize::MAX..usize::MAX;
            }
        }
        if is_last_node_in_parent && needs_to_replace {
            current_range.end = formatter.len();
            ranges_to_replace.push((current_range.clone(), whitespaces));
        }

        for (range, whitespaces) in ranges_to_replace.iter().rev() {
            let replacement: Vec<S::CodeUnit> =
                (0..*whitespaces).map(|_| nbsp.clone()).collect();
            formatter.write_at_range(range.clone(), &replacement);
        }
    }
}

impl<S> ToHtml<S> for TextNode<S>
where
    S: UnicodeString,
{
    fn fmt_html(
        &self,
        f: &mut HtmlFormatter<S>,
        selection_writer: Option<&mut SelectionWriter>,
        is_last_node_in_parent: bool,
    ) {
        let cur_pos = f.len();
        let string = self.data.to_utf8();
        let escaped = html_escape::encode_text(&string);
        f.write(S::from_str(&escaped).as_slice());
        Self::handle_several_whitespaces(f, cur_pos, is_last_node_in_parent);
        if let Some(selection_writer) = selection_writer {
            selection_writer.write_selection_text_node(f, cur_pos, self);
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
        let mut description = S::from_str("\"");
        let text = &self.data.to_utf8().replace('\u{200b}', "~");
        description.push_string(&S::from_str(text));
        description.push_string(&S::from_str("\""));
        return self.tree_line(
            description,
            self.handle.raw().len(),
            continuous_positions,
        );
    }
}
