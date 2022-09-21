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
use std::marker::PhantomData;

use crate::dom::dom_handle::DomHandle;
use crate::dom::html_formatter::HtmlFormatter;
use crate::dom::to_html::ToHtml;
use crate::dom::to_raw_text::ToRawText;
use crate::dom::to_tree::ToTree;
use crate::dom::{HtmlChar, UnicodeString};

#[derive(Clone, Debug, PartialEq)]
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
        S::from_str("br")
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
        f: &mut HtmlFormatter<S>,
        _: Option<&mut SelectionWriter>,
    ) {
        f.write_char(HtmlChar::Lt);
        f.write(self.name().as_slice());
        f.write_char(HtmlChar::Space);
        f.write_char(HtmlChar::ForwardSlash);
        f.write_char(HtmlChar::Gt);
    }
}

impl<S> ToRawText<S> for LineBreakNode<S>
where
    S: UnicodeString,
{
    fn to_raw_text(&self) -> S {
        S::from_str("\\n")
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
            continuous_positions.clone(),
        )
    }
}
