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

use crate::dom::{Dom, UnicodeString};
use crate::{InlineFormatType, Location};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ComposerState<S>
where
    S: UnicodeString,
{
    pub dom: Dom<S>,
    pub start: Location,
    pub end: Location,
    pub toggled_format_types: Vec<InlineFormatType>,
}

impl<S> ComposerState<S>
where
    S: UnicodeString,
{
    pub fn new() -> Self {
        Self {
            dom: Dom::default(),
            start: Location::default(),
            end: Location::default(),
            toggled_format_types: Vec::new(),
        }
    }

    pub fn advance_selection(&mut self) {
        self.start += 1;
        self.end += 1;
    }

    /// Extends the selection by the given number of code points by moving the
    /// greater of the two selection points.
    ///
    pub(crate) fn extend_selection(&mut self, length: isize) {
        if self.start > self.end {
            self.start += length;
        } else {
            self.end += length;
        }
    }
}
