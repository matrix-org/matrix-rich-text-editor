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

use super::{
    nodes::dom_node::DomNodeKind, unicode_string::UnicodeStringExt,
    UnicodeString,
};

pub trait ToHtml<S>
where
    S: UnicodeString,
{
    /// Convert to HTML
    ///
    /// When `is_message` is true, it outputs a clean representation of the
    /// source object, suitable for sending as a message.
    fn fmt_html(
        &self,
        buf: &mut S,
        selection_writer: Option<&mut SelectionWriter>,
        state: &ToHtmlState,
        as_message: bool,
    );

    /// Convert to a clean HTML represention of the source object, suitable
    /// for sending as a message
    fn to_message_html(&self) -> S {
        let mut buf = S::default();
        self.fmt_html(&mut buf, None, &ToHtmlState::default(), true);
        buf
    }

    /// Convert to a literal HTML represention of the source object
    fn to_html(&self) -> S {
        let mut buf = S::default();
        self.fmt_html(&mut buf, None, &ToHtmlState::default(), false);
        buf
    }
}

pub trait ToHtmlExt<S>: ToHtml<S>
where
    S: UnicodeString,
{
    fn fmt_tag_open(
        &self,
        name: &S::Str,
        formatter: &mut S,
        attrs: &Option<Vec<(S, S)>>,
    );
    fn fmt_tag_close(&self, name: &S::Str, formatter: &mut S);
}

impl<S, H: ToHtml<S>> ToHtmlExt<S> for H
where
    S: UnicodeString,
{
    fn fmt_tag_close(&self, name: &S::Str, formatter: &mut S) {
        formatter.push("</");
        formatter.push(name);
        formatter.push('>');
    }

    fn fmt_tag_open(
        &self,
        name: &S::Str,
        formatter: &mut S,
        attrs: &Option<Vec<(S, S)>>,
    ) {
        formatter.push('<');
        formatter.push(name);
        if let Some(attrs) = attrs {
            for attr in attrs {
                let (attr_name, value) = attr;
                formatter.push(' ');
                formatter.push(&**attr_name);
                formatter.push("=\"");
                formatter.push(&**value);
                formatter.push('"');
            }
        }
        formatter.push('>');
    }
}

/// State of the HTML generation at every `fmt_html` call, usually used to pass info from ancestor
/// nodes to their descendants.
#[derive(Clone, Default)]
pub struct ToHtmlState {
    pub is_inside_code_block: bool,
    pub prev_sibling: Option<DomNodeKind>,
    pub next_sibling: Option<DomNodeKind>,
}
