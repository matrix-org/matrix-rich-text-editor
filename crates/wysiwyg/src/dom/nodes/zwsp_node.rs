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
use crate::dom::to_html::ToHtmlState;
use crate::dom::to_markdown::MarkdownOptions;
use crate::dom::unicode_string::{UnicodeStrExt, UnicodeStringExt};
use crate::{
    DomHandle, MarkdownError, ToHtml, ToMarkdown, ToRawText, ToTree,
    UnicodeString,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZwspNode<S>
where
    S: UnicodeString,
{
    data: S,
    handle: DomHandle,
}

impl<S> Default for ZwspNode<S>
where
    S: UnicodeString,
{
    /// Create a new default ZwspNode
    ///
    /// NOTE: Its handle() will be unset until you call set_handle() or
    /// append() it to another node.
    fn default() -> Self {
        Self {
            data: S::zwsp(),
            handle: DomHandle::new_unset(),
        }
    }
}

impl<S> ZwspNode<S>
where
    S: UnicodeString,
{
    pub fn data(&self) -> &S::Str {
        &self.data
    }

    pub fn handle(&self) -> DomHandle {
        self.handle.clone()
    }

    pub fn set_handle(&mut self, handle: DomHandle) {
        self.handle = handle;
    }
}

impl<S> ToHtml<S> for ZwspNode<S>
where
    S: UnicodeString,
{
    fn fmt_html(
        &self,
        buf: &mut S,
        selection_writer: Option<&mut SelectionWriter>,
        _: ToHtmlState,
    ) {
        let cur_pos = buf.len();
        buf.push(self.data());

        if let Some(selection_writer) = selection_writer {
            selection_writer.write_selection_zwsp_node(buf, cur_pos, self)
        }
    }
}

impl<S> ToMarkdown<S> for ZwspNode<S>
where
    S: UnicodeString,
{
    fn fmt_markdown(
        &self,
        buffer: &mut S,
        _options: &MarkdownOptions,
    ) -> Result<(), MarkdownError<S>> {
        buffer.push(self.data.to_owned());

        Ok(())
    }
}

impl<S> ToRawText<S> for ZwspNode<S>
where
    S: UnicodeString,
{
    fn to_raw_text(&self) -> S {
        self.data.clone()
    }
}

impl<S> ToTree<S> for ZwspNode<S>
where
    S: UnicodeString,
{
    fn to_tree_display(&self, continuous_positions: Vec<usize>) -> S {
        self.tree_line(
            "~".into(),
            self.handle.raw().len(),
            continuous_positions,
        )
    }
}
