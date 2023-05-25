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

use std::ops::ControlFlow;

use crate::char::CharExt;
use crate::composer_model::example_format::SelectionWriter;
use crate::dom::dom_handle::DomHandle;
use crate::dom::nodes::dom_node::{DomNode, DomNodeKind};
use crate::dom::to_html::{ToHtml, ToHtmlState};
use crate::dom::to_markdown::{MarkdownError, MarkdownOptions, ToMarkdown};
use crate::dom::to_plain_text::ToPlainText;
use crate::dom::to_raw_text::ToRawText;
use crate::dom::to_tree::ToTree;
use crate::dom::unicode_string::{UnicodeStr, UnicodeStrExt, UnicodeStringExt};
use crate::dom::{self, UnicodeString};
use crate::{InlineFormatType, ListType};

#[derive(Clone, Debug, PartialEq)]
pub struct MentionNode<S>
where
    S: UnicodeString,
{
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
        // do the things we need to do for all cases - add the required attributes and create a handle
        attributes.push(("href".into(), url.clone()));
        attributes.push(("contenteditable".into(), "false".into()));
        let handle = DomHandle::new_unset();

        // for now, we're going to check the display_text and attributes to figure out which
        // mention to build - this is a bit hacky and may change in the future when we
        // can infer the type directly from the url
        if display_text == "@room".into() {
            return Self {
                display_text,
                kind: MentionNodeKind::AtRoom,
                attrs: attributes,
                // I _think_ this is the best way to handle it, can replace this with a # placeholder
                // as that's how you make a placeholder link in html
                url: None,
                handle,
            };
        }

        let kind = if attributes
            .contains(&(S::from("data-mention-type"), S::from("user")))
        {
            MentionNodeKind::User
        } else {
            MentionNodeKind::Room
        };

        Self {
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
    pub fn name(&self) -> S {
        "a".into()
    }

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
