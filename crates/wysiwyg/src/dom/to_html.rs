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

use super::UnicodeString;

pub trait ToHtml<S>
where
    S: UnicodeString,
{
    fn fmt_html(
        &self,
        buf: &mut S,
        selection_writer: Option<&mut SelectionWriter>,
        state: ToHtmlState,
    );

    fn to_html(&self) -> S {
        let mut buf = S::default();
        self.fmt_html(&mut buf, None, ToHtmlState::default());
        buf
    }
}

/// State of the HTML generation at every `fmt_html` call, usually used to pass info from ancestor
/// nodes to their descendants.
#[derive(Copy, Clone, Default)]
pub struct ToHtmlState {
    pub is_inside_code_block: bool,
    pub is_last_node_in_parent: bool,
    pub is_first_node_in_parent: bool,
}
