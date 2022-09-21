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
use std::cmp::Ordering;

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

    /// Remember the values passed in when were were created, so we can
    /// recreate this SameNodeRange as a MultipleNodesRange. This will help
    /// with our transition to only using MultipleNodesRange.
    pub original_start: usize,
    pub original_end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomLocation {
    pub node_handle: DomHandle,
    pub position: usize,
    pub start_offset: usize,
    pub end_offset: usize,
    pub length: usize,
    pub is_leaf: bool,
}

impl DomLocation {
    pub fn new(
        node_handle: DomHandle,
        position: usize,
        start_offset: usize,
        end_offset: usize,
        length: usize,
        is_leaf: bool,
    ) -> Self {
        Self {
            node_handle,
            position,
            start_offset,
            end_offset,
            length,
            is_leaf,
        }
    }

    /// Calculated index in the Dom based on the [position] and [start_offset] values.
    pub fn index_in_dom(&self) -> usize {
        self.position + self.start_offset
    }

    /// Create a copy of this Location, but with start and end offsets reversed
    pub fn reversed(&self) -> Self {
        Self {
            node_handle: self.node_handle.clone(),
            position: self.position,
            start_offset: self.end_offset,
            end_offset: self.start_offset,
            length: self.length,
            is_leaf: self.is_leaf,
        }
    }

    /// Whether the selection starts at this location or not
    pub fn is_start(&self) -> bool {
        self.end_offset == self.length
    }

    /// Whether the selection ends at this location or not
    pub fn is_end(&self) -> bool {
        self.start_offset == 0
    }

    /// Whether the selection completely covers this location
    pub fn is_covered(&self) -> bool {
        self.is_start() && self.is_end()
    }
}

impl PartialOrd<Self> for DomLocation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DomLocation {
    fn cmp(&self, other: &Self) -> Ordering {
        self.node_handle.cmp(&other.node_handle)
    }
}

#[derive(Debug, PartialEq, Clone)]
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

    /// Return the position of the first character in this range.
    /// The position is measured in code units.
    /// If the range starts at the beginning of the Dom, the return value is 0.
    /// If this range has zero length, the position returned is the position
    /// of both the beginning and the end.
    pub fn start(&self) -> usize {
        // Assumes leaf locations are in order, so the first leaf we hit will
        // be the earliest in the Dom.

        self.locations
            .iter()
            .find_map(|loc| {
                if loc.is_leaf {
                    Some(loc.position + loc.start_offset)
                } else {
                    None
                }
            })
            .unwrap_or(0)
    }
}

impl IntoIterator for MultipleNodesRange {
    type Item = DomLocation;
    type IntoIter = std::vec::IntoIter<DomLocation>;

    fn into_iter(self) -> Self::IntoIter {
        self.locations.into_iter()
    }
}

#[cfg(test)]
mod test {
    use crate::tests::testutils_composer_model::cm;

    use super::{MultipleNodesRange, Range};

    #[test]
    fn range_start_for_cursor_at_beginning() {
        assert_eq!(range_of("|abc<b>def</b>").start(), 0);
    }

    #[test]
    fn range_start_for_selection_at_beginning() {
        assert_eq!(range_of("{abc<b>d}|ef</b>").start(), 0);
    }

    #[test]
    fn range_start_for_cursor_in_middle_of_plain_text() {
        assert_eq!(range_of("abc|def").start(), 3);
    }

    #[test]
    fn range_start_for_selection_in_middle_of_plain_text() {
        assert_eq!(range_of("abc{def}|ghi").start(), 3);
    }

    #[test]
    fn range_start_for_cursor_in_nested_tags() {
        assert_eq!(
            range_of(
                "\
                <ul><li>a</li><li>b</li></ul>\
                <ul><li>c</li><li>d|</li><li>e</li></ul>"
            )
            .start(),
            4
        );
    }

    #[test]
    fn range_start_for_selection_in_nested_tags() {
        assert_eq!(
            range_of(
                "\
                <ul><li>a</li><li>b</li></ul>\
                <ul><li>c</li><li>{d</li><li>e}|</li></ul>"
            )
            .start(),
            3
        );
    }

    #[test]
    fn range_start_for_end_of_complex_tags() {
        assert_eq!(
            range_of(
                "\
                <ul><li>a</li><li>b</li></ul>\
                <ul><li>c</li><li>d</li><li>e|</li></ul>"
            )
            .start(),
            5
        );
    }

    #[test]
    fn range_start_for_end_of_text() {
        assert_eq!(
            range_of(
                "\
                <ul><li>a</li><li>b</li></ul>\
                <ul><li>c</li><li>d</li><li>e</li></ul>fgh|"
            )
            .start(),
            8
        );
    }

    fn range_of(model: &str) -> MultipleNodesRange {
        let model = cm(model);
        let (s, e) = model.safe_selection();
        let range = model.state.dom.find_range(s, e);
        match range {
            Range::SameNode(range) => {
                model.state.dom.convert_same_node_range_to_multi(range)
            }
            Range::MultipleNodes(mrange) => mrange,
            Range::NoNode => panic!("Wasn't expecting NoNode!"),
        }
    }
}
