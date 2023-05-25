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
    href: S,
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
}
