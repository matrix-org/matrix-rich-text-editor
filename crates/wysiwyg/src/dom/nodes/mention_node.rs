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
use matrix_mentions::{get_at_room_display_text, Mention, MentionKind};

use crate::composer_model::example_format::SelectionWriter;
use crate::dom::dom_handle::DomHandle;
use crate::dom::to_html::{ToHtml, ToHtmlExt, ToHtmlState};
use crate::dom::to_markdown::{MarkdownError, MarkdownOptions, ToMarkdown};
use crate::dom::to_plain_text::ToPlainText;
use crate::dom::to_raw_text::ToRawText;
use crate::dom::to_tree::ToTree;
use crate::dom::unicode_string::{UnicodeStrExt, UnicodeStringExt};
use crate::dom::UnicodeString;

#[derive(Debug)]
pub struct UriParseError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MentionNode<S>
where
    S: UnicodeString,
{
    // `display_text` refers to that passed by the client which may, in some cases, be different
    // from the ruma derived `Mention.display_text`
    display_text: S,
    kind: MentionNodeKind,
    attributes: Vec<(S, S)>,
    handle: DomHandle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MentionNodeKind {
    MatrixURI { mention: Mention },
    AtRoom,
}

impl<S> MentionNode<S>
where
    S: UnicodeString,
{
    /// Create a new MentionNode. This may fail if the uri can not be parsed, so
    /// it will return `Result<MentionNode>`
    ///
    /// NOTE: Its handle() will be unset until you call set_handle() or
    /// append() it to another node.
    pub fn new(
        url: S,
        display_text: S,
        attributes: Vec<(S, S)>,
    ) -> Result<Self, UriParseError> {
        let handle = DomHandle::new_unset();

        if let Some(mention) = Mention::from_uri_with_display_text(
            &url.to_string(),
            &display_text.to_string(),
        ) {
            let kind = MentionNodeKind::MatrixURI { mention };
            Ok(Self {
                display_text,
                kind,
                attributes,
                handle,
            })
        } else {
            Err(UriParseError)
        }
    }

    /// Create a new at-room MentionNode.
    ///
    /// NOTE: Its handle() will be unset until you call set_handle() or
    /// append() it to another node.
    pub fn new_at_room(attributes: Vec<(S, S)>) -> Self {
        let handle = DomHandle::new_unset();

        Self {
            display_text: S::from(get_at_room_display_text()),
            kind: MentionNodeKind::AtRoom,
            attributes,
            handle,
        }
    }

    pub fn name(&self) -> S {
        S::from("mention")
    }

    pub fn display_text(&self) -> S {
        match self.kind() {
            MentionNodeKind::MatrixURI { .. } => self.display_text.clone(),
            MentionNodeKind::AtRoom => S::from(get_at_room_display_text()),
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

    pub fn kind(&self) -> &MentionNodeKind {
        &self.kind
    }

    // /// Util function to check if the display text is that of an at-room mention
    // pub fn is_at_room_display_text(text: &S) -> bool {
    //     text == &S::from(AT_ROOM)
    // }

    // /// Util function to get the display text for an at-room mention
    // pub fn get_at_room_display_text() -> S {
    //     S::from(AT_ROOM)
    // }
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
            MentionNodeKind::MatrixURI { mention } => {
                // if formatting as a message, only include the href attribute
                let attributes = if as_message {
                    vec![("href".into(), S::from(mention.uri()))]
                } else {
                    let mut attrs = self.attributes.clone();
                    attrs.push(("href".into(), S::from(mention.uri())));
                    attrs.push(("contenteditable".into(), "false".into()));
                    attrs
                };

                let display_text =
                    if as_message && mention.kind() == &MentionKind::Room {
                        S::from(mention.mx_id())
                    } else {
                        self.display_text()
                    };

                self.fmt_tag_open(tag, formatter, &Some(attributes));
                formatter.push(display_text);
                self.fmt_tag_close(tag, formatter);
            }
            MentionNodeKind::AtRoom => {
                // if formatting as a message, simply use the display text (@room)
                if as_message {
                    formatter.push(self.display_text())
                } else {
                    let mut attributes = self.attributes.clone();
                    attributes.push(("href".into(), "#".into())); // designates a placeholder link in html
                    attributes.push(("contenteditable".into(), "false".into()));

                    self.fmt_tag_open(tag, formatter, &Some(attributes));
                    formatter.push(self.display_text());
                    self.fmt_tag_close(tag, formatter);
                };
            }
        }

        if let Some(sel_writer) = selection_writer {
            sel_writer.write_selection_mention_node(formatter, cur_pos, self);
        }
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
        description.push("\"");

        match self.kind() {
            MentionNodeKind::MatrixURI { mention } => {
                description.push(", ");
                description.push(S::from(mention.uri()));
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
        fmt_mention(self, buffer)?;
        return Ok(());

        #[inline(always)]
        fn fmt_mention<S>(
            this: &MentionNode<S>,
            buffer: &mut S,
        ) -> Result<(), MarkdownError<S>>
        where
            S: UnicodeString,
        {
            let text = match this.kind() {
                // for User/Room type, we use the mx_id in the md output
                MentionNodeKind::MatrixURI { mention } => {
                    if mention.kind() == &MentionKind::Room {
                        S::from(mention.mx_id())
                    } else {
                        this.display_text()
                    }
                }
                MentionNodeKind::AtRoom => this.display_text(),
            };

            buffer.push(text);
            Ok(())
        }
    }
}
