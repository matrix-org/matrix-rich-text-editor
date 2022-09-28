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

/// Represents a part of a Range.
/// This is made up of a node (we hold a handle to it), and which part of
/// that node is within the range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomLocation {
    /// The handle of the node we are talking about
    pub node_handle: DomHandle,

    /// The position inside this node of the start of the range. In a text
    /// node this will be the number of code points through the text to
    /// get to the start of the range. In a container node this will be how
    /// far through children nodes you need to count to get to the start.
    /// In a text-like node like a line break, this will be 0 or 1.
    /// Measured in code units.
    pub start_offset: usize,

    /// The position inside this node of the end of the range. In a text
    /// node this will be the number of code points through the text to
    /// get to the end of the range. In a container node this will be how
    /// far through children nodes you need to count to get to the end.
    /// In a text-like node like a line break, this will be 0 or 1.
    /// Measured in code units.
    pub end_offset: usize,

    /// Where within the whole Dom is this node? Measured in code units.
    pub position: usize,

    /// How many code units are inside this node? In a text node this will
    /// be the length of the text, and in a container node this will be the
    /// sum of the lengths of the contained nodes. In a text-like node like
    /// a line break, this will be 1.
    pub length: usize,

    /// True if this is a node which is not a container i.e. a text node or
    /// a text-like node like a line break.
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

    pub fn with_new_handle(&self, handle: DomHandle) -> Self {
        Self {
            node_handle: handle,
            position: usize::MAX, // Position is no longer valid
            start_offset: self.start_offset,
            end_offset: self.end_offset,
            length: self.length,
            is_leaf: self.is_leaf,
        }
    }

    /// Calculated index in the Dom based on the [position] and [start_offset]
    /// values.
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
        let end_offset = if self.start_offset < self.end_offset {
            self.end_offset
        } else {
            self.start_offset
        };
        end_offset == self.length
    }

    /// Whether the selection ends at this location or not
    pub fn is_end(&self) -> bool {
        let start_offset = if self.start_offset < self.end_offset {
            self.start_offset
        } else {
            self.end_offset
        };
        start_offset == 0
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
pub struct Range {
    pub locations: Vec<DomLocation>,
}

impl Range {
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

    pub fn end(&self) -> usize {
        self.locations
            .iter()
            .rev()
            .find_map(|loc| {
                if loc.is_leaf {
                    Some(loc.position + loc.end_offset)
                } else {
                    None
                }
            })
            .unwrap_or(0)
    }

    pub fn leaves(&self) -> impl Iterator<Item = &DomLocation> {
        self.locations.iter().filter(|loc| loc.is_leaf)
    }

    // TODO: remove all uses of this when we guarantee that Dom is never empty
    pub fn is_empty(&self) -> bool {
        self.locations.is_empty()
    }
}

impl IntoIterator for Range {
    type Item = DomLocation;
    type IntoIter = std::vec::IntoIter<DomLocation>;

    fn into_iter(self) -> Self::IntoIter {
        self.locations.into_iter()
    }
}

#[cfg(test)]
mod test {
    use crate::{
        dom::DomLocation, tests::testutils_composer_model::cm, DomHandle,
    };

    use super::Range;

    #[test]
    fn range_start_and_end_for_cursor_at_beginning() {
        let r = range_of("|abc<b>def</b>");
        assert_eq!(r.start(), 0);
        assert_eq!(r.end(), 0);
    }

    #[test]
    fn range_start_and_end_for_selection_at_beginning() {
        let r = range_of("{abc<b>d}|ef</b>");
        assert_eq!(r.start(), 0);
        assert_eq!(r.end(), 4);
    }

    #[test]
    fn range_start_and_end_for_cursor_after_brs() {
        let r = range_of("a<br />b<br /><br />c|<br />d");
        assert_eq!(r.start(), 6);
        assert_eq!(r.end(), 6);
    }

    #[test]
    fn range_start_and_end_for_selection_containing_brs_in_tags() {
        let r = range_of("<i>a</i><b><br />{b<br /><br />c}|<br /></b>d");
        assert_eq!(r.start(), 2);
        assert_eq!(r.end(), 6);
    }

    #[test]
    fn range_start_and_end_for_cursor_in_middle_of_plain_text() {
        let r = range_of("abc|def");
        assert_eq!(r.start(), 3);
        assert_eq!(r.end(), 3);
    }

    #[test]
    fn range_leaves_contains_text_node() {
        let r = range_of("abc|def");
        assert_eq!(
            r.leaves().collect::<Vec<&DomLocation>>(),
            vec![&r.locations[0]]
        );
    }

    #[test]
    fn range_start_and_end_for_selection_in_middle_of_plain_text() {
        let r = range_of("abc{def}|ghi");
        assert_eq!(r.start(), 3);
        assert_eq!(r.end(), 6);
    }

    #[test]
    fn range_start_and_end_for_cursor_in_nested_tags() {
        let r = range_of(
            "\
            <ul><li>a</li><li>b</li></ul>\
            <ul><li>c</li><li>d|</li><li>e</li></ul>",
        );
        assert_eq!(r.start(), 4);
        assert_eq!(r.end(), 4);
    }

    #[test]
    fn range_start_and_end_for_selection_in_nested_tags() {
        let r = range_of(
            "\
            <ul><li>a</li><li>b</li></ul>\
            <ul><li>c</li><li>{d</li><li>e}|</li></ul>",
        );

        assert_eq!(r.start(), 3);
        assert_eq!(r.end(), 5);
    }

    #[test]
    fn range_start_and_end_for_end_of_complex_tags() {
        let r = range_of(
            "\
            <ul><li>a</li><li>b</li></ul>\
            <ul><li>c</li><li>d</li><li>e|</li></ul>",
        );

        assert_eq!(r.start(), 5);
        assert_eq!(r.end(), 5);
    }

    #[test]
    fn range_start_and_end_for_end_of_text() {
        let r = range_of(
            "\
            <ul><li>a</li><li>b</li></ul>\
            <ul><li>c</li><li>d</li><li>e</li></ul>fgh|",
        );

        assert_eq!(r.start(), 8);
        assert_eq!(r.end(), 8);
    }

    #[test]
    fn range_leaves_contains_all_text_nodes() {
        let r = range_of(
            "\
            <ul><li>{a</li><li>b</li></ul>\
            <ul><li>c</li><li>d</li><li>e</li></ul>fgh}|",
        );

        assert_eq!(
            r.leaves()
                .map(|loc| &loc.node_handle)
                .cloned()
                .collect::<Vec<DomHandle>>(),
            vec![
                DomHandle::from_raw(vec![0, 0, 0]), // a
                DomHandle::from_raw(vec![0, 1, 0]), // b
                DomHandle::from_raw(vec![1, 0, 0]), // c
                DomHandle::from_raw(vec![1, 1, 0]), // d
                DomHandle::from_raw(vec![1, 2, 0]), // e
                DomHandle::from_raw(vec![2]),       // fgh
            ]
        );
    }

    fn range_of(model: &str) -> Range {
        let model = cm(model);
        let (s, e) = model.safe_selection();
        model.state.dom.find_range(s, e)
    }
}
