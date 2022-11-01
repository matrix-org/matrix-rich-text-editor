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
use crate::dom::to_html::ToHtml;
use crate::dom::to_markdown::{MarkdownError, MarkdownOptions, ToMarkdown};
use crate::dom::to_raw_text::ToRawText;
use crate::dom::to_tree::ToTree;
use crate::dom::unicode_string::{UnicodeStrExt, UnicodeStringExt};
use crate::dom::UnicodeString;
use std::marker::PhantomData;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineBreakNode<S>
where
    S: UnicodeString,
{
    _phantom_data: PhantomData<S>,
    handle: DomHandle,
}

impl<S> LineBreakNode<S>
where
    S: UnicodeString,
{
    /// Create a new LineBreakNode
    ///
    /// NOTE: Its handle() will be unset until you call set_handle() or
    /// append() it to another node.
    pub fn new() -> Self {
        Self {
            _phantom_data: PhantomData {},
            handle: DomHandle::new_unset(),
        }
    }

    pub fn name(&self) -> S {
        "br".into()
    }

    pub fn handle(&self) -> DomHandle {
        self.handle.clone()
    }

    pub fn set_handle(&mut self, handle: DomHandle) {
        self.handle = handle;
    }

    // A br tag is always treated as 1 character, so this always returns 1
    pub fn text_len(&self) -> usize {
        1
    }
}

impl<S> ToHtml<S> for LineBreakNode<S>
where
    S: UnicodeString,
{
    fn fmt_html(
        &self,
        buf: &mut S,
        selection_writer: Option<&mut SelectionWriter>,
        _: bool,
    ) {
        let cur_pos = buf.len();
        buf.push('<');
        buf.push(self.name());
        buf.push(' ');
        buf.push('/');
        buf.push('>');
        if let Some(sel_writer) = selection_writer {
            sel_writer.write_selection_line_break_node(buf, cur_pos, self);
        }
    }
}

impl<S> ToRawText<S> for LineBreakNode<S>
where
    S: UnicodeString,
{
    fn to_raw_text(&self) -> S {
        "\\n".into()
    }
}

impl<S> ToTree<S> for LineBreakNode<S>
where
    S: UnicodeString,
{
    fn to_tree_display(&self, continuous_positions: Vec<usize>) -> S {
        self.tree_line(
            self.name(),
            self.handle.raw().len(),
            continuous_positions,
        )
    }
}

impl<S> ToMarkdown<S> for LineBreakNode<S>
where
    S: UnicodeString,
{
    fn fmt_markdown(
        &self,
        buffer: &mut S,
        options: &MarkdownOptions,
    ) -> Result<(), MarkdownError<S>> {
        if options.contains(MarkdownOptions::IGNORE_LINE_BREAK) {
            // Replace the line break by a single space.
            buffer.push(' ');
        } else {
            // A line break is a `\n` in Markdown. Two or more line breaks
            // usually generate a new block (i.e. a new paragraph). To
            // avoid that, we can prefix `\n` by a backslash. Thus:
            //
            // ```html
            // abc<br />def
            //
            // ghi<br /><br />jkl
            // ```
            //
            // maps to:
            //
            // ```md
            // abc\
            // def
            //
            // ghi\
            // \
            // jkl
            // ```
            buffer.push("\\\n");
        }

        Ok(())
    }
}
