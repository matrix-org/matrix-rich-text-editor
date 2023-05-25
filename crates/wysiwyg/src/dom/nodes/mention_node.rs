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
    name: S,
    display_text: S,
    kind: MentionNodeKind,
    attrs: Vec<(S, S)>,
    url: Option<S>,
    handle: DomHandle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MentionNodeKind {
    User,
    Room,
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
        // do the things we need to do for all cases - add the contenteditable attribute and create a handle and name
        attributes.push(("contenteditable".into(), "false".into()));
        let handle = DomHandle::new_unset();
        let name = "a".into();

        // for now, we're going to check the display_text and attributes to figure out which
        // mention to build - this is a bit hacky and may change in the future when we
        // can infer the type directly from the url
        if display_text == "@room".into() {
            // we set a placeholder here to ensure semantic html in the output
            attributes.push(("href".into(), "#".into()));
            return Self {
                name,
                display_text,
                kind: MentionNodeKind::AtRoom,
                attrs: attributes,
                url: None,
                handle,
            };
        }

        attributes.push(("href".into(), url.clone()));

        let kind = if attributes
            .contains(&(S::from("data-mention-type"), S::from("user")))
        {
            MentionNodeKind::User
        } else {
            MentionNodeKind::Room
        };

        Self {
            name,
            display_text,
            kind,
            attrs: attributes,
            url: Some(url),
            handle,
        }
    }

    /**
     * LIFTED FROM LINE_BREAK_NODE.RS
     */

    pub fn set_handle(&mut self, handle: DomHandle) {
        self.handle = handle;
    }

    pub fn handle(&self) -> DomHandle {
        self.handle.clone()
    }

    pub fn text_len(&self) -> usize {
        self.display_text.len()
    }

    /**
     * LIFTED FROM CONTAINER_NODE.RS
     */
    pub fn name(&self) -> &S::Str {
        &self.name
    }

    pub fn attributes(&self) -> &Vec<(S, S)> {
        self.attrs.as_ref()
    }

    pub fn kind(&self) -> &MentionNodeKind {
        &self.kind
    }
    pub(crate) fn get_mention_url(&self) -> Option<S> {
        self.url.clone()
    }

    /// Returns true if the ContainerNode has no children.
    pub fn is_empty(&self) -> bool {
        self.display_text.len() == 0
    }

    /// Returns true if there is no text in this ContainerNode.
    pub fn has_no_text(&self) -> bool {
        self.display_text.len() == 0
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
    ) {
        self.fmt_mention_html(formatter, selection_writer, state)
    }
}

impl<S: UnicodeString> MentionNode<S> {
    fn fmt_mention_html(
        &self,
        formatter: &mut S,
        _: Option<&mut SelectionWriter>,
        _: ToHtmlState,
    ) {
        assert!(matches!(
            self.kind,
            MentionNodeKind::Room
                | MentionNodeKind::User
                | MentionNodeKind::AtRoom
        ));

        let name = self.name();
        self.fmt_tag_open(name, formatter, self.attrs.clone());

        formatter.push(self.display_text.clone());

        self.fmt_tag_close(name, formatter);
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
        // no idea if this is correct
        self.display_text.clone()
    }
}

impl<S> ToPlainText<S> for MentionNode<S>
where
    S: UnicodeString,
{
    fn to_plain_text(&self) -> S {
        // no idea if this is correct
        self.display_text.clone()
    }
}

impl<S> ToTree<S> for MentionNode<S>
where
    S: UnicodeString,
{
    fn to_tree_display(&self, continuous_positions: Vec<usize>) -> S {
        let mut description = self.name.clone();

        if let Some(url) = &self.url {
            description.push(" \"");
            description.push(url.clone());
            description.push("\"");
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
            User | Room => {
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
            buffer.push(this.display_text.clone());
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
            // should this be "@room".into()? not sure what's clearer
            buffer.push(this.display_text.clone());
            Ok(())
        }
    }
}
