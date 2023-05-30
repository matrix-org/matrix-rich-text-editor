// Copyright 2023 The Matrix.org Foundation C.I.C.
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
use crate::dom::to_html::{ToHtml, ToHtmlState};
use crate::dom::to_markdown::{MarkdownError, MarkdownOptions, ToMarkdown};
use crate::dom::to_plain_text::ToPlainText;
use crate::dom::to_raw_text::ToRawText;
use crate::dom::to_tree::ToTree;
use crate::dom::unicode_string::{UnicodeStrExt, UnicodeStringExt};
use crate::dom::UnicodeString;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MentionNode<S>
where
    S: UnicodeString,
{
    kind: MentionNodeKind<S>,
    handle: DomHandle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MentionNodeKind<S>
where
    S: UnicodeString,
{
    MatrixUrl { display_text: S, url: S },
    AtRoom,
}

impl<S> MentionNode<S>
where
    S: UnicodeString,
{
    /// Create a new MentionNode
    ///
    /// NOTE: Its handle() will be unset until you call set_handle() or
    /// append() it to another node.
    pub fn new(url: S, display_text: S, mut attributes: Vec<(S, S)>) -> Self {
        let handle = DomHandle::new_unset();

        // TODO: do something with the attributes

        Self {
            kind: MentionNodeKind::MatrixUrl { display_text, url },
            handle,
        }
    }

    pub fn new_at_room() -> Self {
        let handle = DomHandle::new_unset();

        Self {
            kind: MentionNodeKind::AtRoom,
            handle,
        }
    }

    pub fn name(&self) -> S {
        S::from("mention")
    }

    pub fn display_text(&self) -> S {
        match self.kind() {
            MentionNodeKind::MatrixUrl { display_text, .. } => {
                display_text.clone()
            }
            MentionNodeKind::AtRoom => S::from("@room"),
        }
    }

    pub fn set_handle(&mut self, handle: DomHandle) {
        self.handle = handle;
    }

    pub fn handle(&self) -> DomHandle {
        self.handle.clone()
    }

    pub fn text_len(&self) -> usize {
        // A mention needs to act as a single object rather than mutable
        // text in the editor. So we treat it as having a length of 1.
        1
    }

    pub fn kind(&self) -> &MentionNodeKind<S> {
        &self.kind
    }
}

impl<S> ToHtml<S> for MentionNode<S>
where
    S: UnicodeString,
{
    fn fmt_html(
        &self,
        formatter: &mut S,
        selection_writer: Option<&mut SelectionWriter>,
        state: ToHtmlState,
        as_message: bool,
    ) {
        self.fmt_mention_html(formatter, selection_writer, state, as_message)
    }
}

impl<S: UnicodeString> MentionNode<S> {
    fn fmt_mention_html(
        &self,
        formatter: &mut S,
        selection_writer: Option<&mut SelectionWriter>,
        _: ToHtmlState,
        as_message: bool,
    ) {
        let tag = &S::from("a");

        let cur_pos = formatter.len();
        match self.kind() {
            MentionNodeKind::MatrixUrl { display_text, url } => {
                let mut attributes: Vec<(S, S)> =
                    vec![(S::from("href"), url.clone())];

                if !as_message {
                    attributes
                        .push((S::from("contenteditable"), S::from("false")))
                    // TODO: data-mention-type = "user" | "room"
                }

                self.fmt_tag_open(tag, formatter, attributes);

                formatter.push(display_text.clone());

                self.fmt_tag_close(tag, formatter);
            }
            MentionNodeKind::AtRoom => {
                formatter.push(self.display_text());
            }
        }

        if let Some(sel_writer) = selection_writer {
            sel_writer.write_selection_mention_node(formatter, cur_pos, self);
        }
    }

    /**
     * LIFTED FROM CONTAINER_NODE.RS
     * TODO could we export/import these to avoid repetition?
     */
    fn fmt_tag_open(
        &self,
        name: &S::Str,
        formatter: &mut S,
        attrs: Vec<(S, S)>,
    ) {
        formatter.push('<');
        formatter.push(name);
        for attr in attrs {
            let (attr_name, value) = attr;
            formatter.push(' ');
            formatter.push(attr_name);
            formatter.push("=\"");
            formatter.push(value);
            formatter.push('"');
        }
        formatter.push('>');
    }

    fn fmt_tag_close(&self, name: &S::Str, formatter: &mut S) {
        formatter.push("</");
        formatter.push(name);
        formatter.push('>');
    }
}

impl<S> ToRawText<S> for MentionNode<S>
where
    S: UnicodeString,
{
    fn to_raw_text(&self) -> S {
        self.display_text()
    }
}

impl<S> ToPlainText<S> for MentionNode<S>
where
    S: UnicodeString,
{
    fn to_plain_text(&self) -> S {
        self.display_text()
    }
}

impl<S> ToTree<S> for MentionNode<S>
where
    S: UnicodeString,
{
    fn to_tree_display(&self, continuous_positions: Vec<usize>) -> S {
        let mut description: S = self.name();

        description.push(" \"");
        description.push(self.display_text());
        description.push(" \"");
        description.push(", ");

        match self.kind() {
            MentionNodeKind::MatrixUrl { url, .. } => {
                description.push(url.clone());
            }
            MentionNodeKind::AtRoom => {}
        }

        let tree_part = self.tree_line(
            description,
            self.handle.raw().len(),
            continuous_positions,
        );

        tree_part
    }
}

impl<S> ToMarkdown<S> for MentionNode<S>
where
    S: UnicodeString,
{
    fn fmt_markdown(
        &self,
        buffer: &mut S,
        _: &MarkdownOptions,
    ) -> Result<(), MarkdownError<S>> {
        use MentionNodeKind::*;

        // There are two different functions to allow for fact one will use mxId later on
        match self.kind() {
            MatrixUrl { .. } => {
                fmt_user_or_room_mention(self, buffer)?;
            }
            AtRoom => {
                fmt_at_room_mention(self, buffer)?;
            }
        }

        return Ok(());

        #[inline(always)]
        fn fmt_user_or_room_mention<S>(
            this: &MentionNode<S>,
            buffer: &mut S,
        ) -> Result<(), MarkdownError<S>>
        where
            S: UnicodeString,
        {
            // TODO make this use mxId, for now we use display_text
            buffer.push(this.display_text());
            Ok(())
        }

        #[inline(always)]
        fn fmt_at_room_mention<S>(
            this: &MentionNode<S>,
            buffer: &mut S,
        ) -> Result<(), MarkdownError<S>>
        where
            S: UnicodeString,
        {
            buffer.push(this.display_text());
            Ok(())
        }
    }
}
