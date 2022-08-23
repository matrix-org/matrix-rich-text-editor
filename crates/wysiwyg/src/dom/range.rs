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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RangeLocationType {
    Start,
    Middle,
    End,
}

#[derive(Debug, PartialEq)]
pub enum Range {
    // The range is within a single node
    SameNode(SameNodeRange),

    // The range covers several nodes
    MultipleNodes(MultipleNodesRange),

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

#[derive(Clone, Debug, PartialEq)]
pub struct DomLocation {
    pub node_handle: DomHandle,
    pub start_offset: usize,
    pub end_offset: usize,
    pub location_type: RangeLocationType,
}

impl DomLocation {
    pub fn new(
        node_handle: DomHandle,
        start_offset: usize,
        end_offset: usize,
        location_type: RangeLocationType,
    ) -> Self {
        Self {
            node_handle,
            start_offset,
            end_offset,
            location_type,
        }
    }

    pub fn reversed(&self) -> Self {
        Self {
            node_handle: self.node_handle.clone(),
            start_offset: self.end_offset,
            end_offset: self.start_offset,
            location_type: self.location_type,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct MultipleNodesRange {
    pub locations: Vec<DomLocation>,
}

impl MultipleNodesRange {
    pub fn new<'a>(
        locations: impl IntoIterator<Item = &'a DomLocation>,
    ) -> Self {
        Self {
            locations: locations.into_iter().cloned().collect(),
        }
    }
}

impl IntoIterator for MultipleNodesRange {
    type Item = DomLocation;
    type IntoIter = std::vec::IntoIter<DomLocation>;

    fn into_iter(self) -> Self::IntoIter {
        self.locations.into_iter()
    }
}
