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
use matrix_mentions::{Mention, MentionKind};

use crate::composer_model::example_format::SelectionWriter;
use crate::dom::dom_handle::DomHandle;
use crate::dom::to_html::{ToHtml, ToHtmlExt, ToHtmlState};
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
    display_text: S,
    kind: MentionNodeKind,
    attributes: Vec<(S, S)>,
    handle: DomHandle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MentionNodeKind {
    Room { mention: Mention },
    User { mention: Mention },
    AtRoom,
    Failed,
}

impl<S> MentionNode<S>
where
    S: UnicodeString,
{
    /// Create a new MentionNode
    ///
    /// NOTE: Its handle() will be unset until you call set_handle() or
    /// append() it to another node.
    pub fn new(url: S, display_text: S, attributes: Vec<(S, S)>) -> Self {
        let handle = DomHandle::new_unset();

        if let Some(mention) = Mention::from_uri_with_display_text(
            &url.to_string(),
            &display_text.to_string(),
        ) {
            let kind = match mention.kind() {
                MentionKind::Room => MentionNodeKind::Room { mention },
                MentionKind::User => MentionNodeKind::User { mention },
            };
            Self {
                display_text,
                kind,
                attributes,
                handle,
            }
        } else {
            Self {
                display_text: S::from("failed to parse"),
                kind: MentionNodeKind::Failed,
                attributes: vec![],
                handle,
            }
        }
    }

    pub fn new_at_room(attributes: Vec<(S, S)>) -> Self {
        let handle = DomHandle::new_unset();

        Self {
            display_text: S::from("@room"),
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
            MentionNodeKind::User { mention } => self.display_text.clone(),
            MentionNodeKind::Room { mention } => self.display_text.clone(),
            MentionNodeKind::AtRoom => S::from("@room"),
            MentionNodeKind::Failed => S::from("failed parsing"),
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
            MentionNodeKind::User { mention } => {
                // if formatting as a message, only include the href attribute
                let attributes = if as_message {
                    vec![("href".into(), S::from(mention.uri()))]
                } else {
                    let mut attributes_for_composer = self.attributes.clone();
                    attributes_for_composer
                        .push(("href".into(), S::from(mention.uri())));
                    attributes_for_composer
                        .push(("contenteditable".into(), "false".into()));
                    attributes_for_composer
                };

                self.fmt_tag_open(tag, formatter, &Some(attributes));
                formatter.push(self.display_text());
                self.fmt_tag_close(tag, formatter);
            }
            MentionNodeKind::Room { mention } => {
                // if formatting as a message, only include the href attribute
                let attributes = if as_message {
                    vec![("href".into(), S::from(mention.uri()))]
                } else {
                    let mut attributes_for_composer = self.attributes.clone();
                    attributes_for_composer
                        .push(("href".into(), S::from(mention.uri())));
                    attributes_for_composer
                        .push(("contenteditable".into(), "false".into()));
                    attributes_for_composer
                };

                self.fmt_tag_open(tag, formatter, &Some(attributes));
                formatter.push(self.display_text());
                self.fmt_tag_close(tag, formatter);
            }
            MentionNodeKind::Failed => {
                // TODO do something
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
            MentionNodeKind::User { mention } => {
                description.push(", ");
                description.push(mention.uri().clone());
            }
            MentionNodeKind::Room { mention } => {
                description.push(", ");
                description.push(mention.uri().clone());
            }
            MentionNodeKind::Failed => {
                description.push(", ");
                description.push("failed parsing");
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
        // There are two different functions to allow for fact one will use mxId later on
        match self.kind() {
            MentionNodeKind::User { mention } => {
                // TODO do something
                fmt_user_or_room_mention(self, mention, buffer)?;
            }
            MentionNodeKind::Room { mention } => {
                // TODO do something
                fmt_user_or_room_mention(self, mention, buffer)?;
            }
            MentionNodeKind::Failed => {
                // TODO do something
            }
            MentionNodeKind::AtRoom => {
                fmt_at_room_mention(self, buffer)?;
            }
        }

        return Ok(());

        #[inline(always)]
        fn fmt_user_or_room_mention<S>(
            this: &MentionNode<S>,
            mention: &Mention,
            buffer: &mut S,
        ) -> Result<(), MarkdownError<S>>
        where
            S: UnicodeString,
        {
            // use the mx_id for a room mention, but the display text (name) for a user
            let text = match this.kind() {
                MentionNodeKind::User { .. } => this.display_text(),
                MentionNodeKind::Room { mention } => S::from(mention.mx_id()),
                _ => S::from("catch all"),
            };

            buffer.push(S::from(text));
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
