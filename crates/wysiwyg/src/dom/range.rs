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

use crate::dom::dom_handle::DomHandle;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum Range {
    SameNode(SameNodeRange),

    // The range is too complex to calculate (for now)
    TooDifficultForMe,

    // The DOM contains no nodes at all!
    NoNode,
}

/// The answer supplied when you ask where a range is in the DOM, and the start
/// and end are both inside the same node.
#[derive(Debug, PartialEq)]
pub struct SameNodeRange {
    /// The node containing the range
    pub node_handle: DomHandle,

    /// The position within this node that corresponds to the start of the range
    pub start_offset: usize,

    /// The position within this node that corresponds to the end of the range
    pub end_offset: usize,
}

impl Display for Range {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Range::SameNode(range) => f.write_str(
                format!(
                    "SameNode: {} {}",
                    range.start_offset, range.end_offset
                )
                .as_str(),
            ),
            Range::TooDifficultForMe => f.write_str("TooDifficultForMe"),
            Range::NoNode => f.write_str("NoNode"),
        }
    }
}
