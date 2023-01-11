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
use crate::dom::nodes::dom_node::DomNodeKind;
use std::cmp::{min, Ordering};

/// Represents the relative position of a DomLocation towards
/// the range start and end.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DomLocationPosition {
    /// Targeted node is before the start of the range (and at the border).
    Before,
    /// Targeted node is after the end of the range (and at the border).
    After,
    /// Targeted node is at least partially within the
    /// range, or cursor is strictly inside the node.
    Inside,
}

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

    /// Node kind
    pub kind: DomNodeKind,
}

impl DomLocation {
    pub fn new(
        node_handle: DomHandle,
        position: usize,
        start_offset: usize,
        end_offset: usize,
        length: usize,
        kind: DomNodeKind,
    ) -> Self {
        Self {
            node_handle,
            position,
            start_offset,
            end_offset,
            length,
            kind,
        }
    }

    pub fn with_new_handle(&self, handle: DomHandle) -> Self {
        Self {
            node_handle: handle,
            position: usize::MAX, // Position is no longer valid
            start_offset: self.start_offset,
            end_offset: self.end_offset,
            length: self.length,
            kind: self.kind.clone(),
        }
    }

    /// True if this is a node which is not a container i.e. a text node or
    /// a text-like node like a line break.
    pub fn is_leaf(&self) -> bool {
        matches!(
            self.kind,
            DomNodeKind::Text | DomNodeKind::LineBreak | DomNodeKind::Zwsp
        )
    }

    /// Returns the relative position of this DomLocation towards the range.
    pub fn relative_position(&self) -> DomLocationPosition {
        if self.length == 0 {
            // Note: this should only trigger on an initial/single empty text node.
            // Other nodes are not allowed to have a length of 0.
            DomLocationPosition::Inside
        } else if self.start_offset == self.length {
            DomLocationPosition::Before
        } else if self.end_offset == 0 {
            DomLocationPosition::After
        } else {
            DomLocationPosition::Inside
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
            kind: self.kind.clone(),
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

    pub fn starts_inside(&self) -> bool {
        self.start_offset > 0
    }

    pub fn ends_inside(&self) -> bool {
        self.end_offset < self.length
    }

    /// Whether the selection completely covers this location
    pub fn is_covered(&self) -> bool {
        self.is_start() && self.is_end()
    }

    /// Allows us to check whether a location is both a list item and the
    /// passed position is at the end of it, we need this to handle adjacent
    /// similar nodes within a list item
    pub fn position_is_end_of_list_item(&self, position: usize) -> bool {
        let is_list_item = self.kind == DomNodeKind::ListItem;

        let location_start = self.position;
        let location_end = self.position + self.length;

        let position_is_at_end_of_location =
            position == location_start || position == location_end;

        is_list_item && position_is_at_end_of_location
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

#[derive(Debug, PartialEq, Eq, Clone)]
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
                if loc.is_leaf() {
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
                if loc.is_leaf() {
                    Some(loc.position + loc.end_offset)
                } else {
                    None
                }
            })
            .unwrap_or(0)
    }

    pub fn leaves(&self) -> impl Iterator<Item = &DomLocation> {
        self.locations.iter().filter(|loc| loc.is_leaf())
    }

    pub fn top_level_depth(&self) -> usize {
        self.locations
            .iter()
            .filter(|l| !l.node_handle.is_root())
            .map(|l| l.node_handle.depth())
            .min()
            .expect("Should always have at least one set handle")
    }

    pub fn locations_at_depth(
        &self,
        depth: usize,
    ) -> impl Iterator<Item = &DomLocation> {
        self.locations
            .iter()
            .filter(move |l| l.node_handle.depth() == depth)
    }

    pub fn locations_from_depth(
        &self,
        depth: usize,
    ) -> impl Iterator<Item = &DomLocation> {
        self.locations
            .iter()
            .filter(move |l| l.node_handle.depth() > depth)
    }

    pub fn top_level_locations(&self) -> impl Iterator<Item = &DomLocation> {
        self.locations_at_depth(self.top_level_depth())
    }

    pub fn node_handles(&self) -> impl DoubleEndedIterator<Item = &DomHandle> {
        self.locations.iter().map(|l| &l.node_handle)
    }

    pub fn is_cursor(&self) -> bool {
        self.start() == self.end()
    }

    pub fn is_selection(&self) -> bool {
        self.start() != self.end()
    }

    // TODO: remove all uses of this when we guarantee that Dom is never empty
    pub fn is_empty(&self) -> bool {
        self.locations.is_empty()
    }

    pub fn contains(&self, handle: &DomHandle) -> bool {
        self.locations.iter().any(|l| l.node_handle == *handle)
    }

    pub(crate) fn find_location(
        &self,
        node_handle: &DomHandle,
    ) -> Option<&DomLocation> {
        self.locations
            .iter()
            .find(|l| *l.node_handle.raw() == *node_handle.raw())
    }

    /// Returns the deepest node that is a parent to all leaves within the
    /// range and is not completely covered by the selection.
    pub fn shared_parent_outside(&self) -> DomHandle {
        self.find_shared_parent(false)
    }

    /// Returns the deepest node that is a parent to all leaves within the
    /// range. May include nodes that are completely covered by the selection.
    pub fn shared_parent(&self) -> DomHandle {
        self.find_shared_parent(true)
    }

    /// Returns the deepest node that is a parent to all leaves within the
    /// range.
    ///
    /// * `allow_covered_nodes` - if true, the node may be covered by the  
    /// selection (i.e. matches the selection).
    ///     
    fn find_shared_parent(&self, allow_covered_nodes: bool) -> DomHandle {
        let mut shared_path = vec![];
        let min_leaf_path = self.leaves().min().unwrap().node_handle.raw();
        let max_leaf_path = self.leaves().max().unwrap().node_handle.raw();

        for i in 0..min(min_leaf_path.len() - 1, max_leaf_path.len() - 1) {
            if min_leaf_path[i] != max_leaf_path[i] {
                break;
            }

            shared_path.push(min_leaf_path[i]);

            if !allow_covered_nodes {
                let location = self
                    .find_location(&DomHandle::from_raw(shared_path.clone()))
                    .expect("Handle was built from leaf node subhandles");

                if location.is_covered() {
                    shared_path.pop();
                    break;
                }
            }
        }

        DomHandle::from_raw(shared_path)
    }

    pub(crate) fn deepest_block_node(
        &self,
        ancestor_of: Option<DomHandle>,
    ) -> Option<&DomLocation> {
        self.locations
            .iter()
            .filter(|l| {
                let mut found = true;
                if let Some(ancestor_of) = &ancestor_of {
                    found = l.node_handle.is_ancestor_of(ancestor_of);
                }
                found && (l.kind.is_block_kind() || l.kind.is_structure_kind())
            })
            .max()
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
        dom::{nodes::dom_node::DomNodeKind, DomLocation},
        tests::testutils_composer_model::cm,
        DomHandle, InlineFormatType,
    };

    use super::{DomLocationPosition, Range};

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
            <ol><li>c</li><li>d</li><li>e</li></ol>fgh}|",
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

    // position_is_end_of_list_item tests
    #[test]
    fn location_method_returns_false_for_end_of_non_list_item() {
        let model = cm("<em>abcd|</em>");
        let (s, e) = model.safe_selection();
        let range = model.state.dom.find_range(s, e);
        assert!(!range.locations[1].position_is_end_of_list_item(4));
    }

    #[test]
    fn location_method_returns_false_for_inside_list_item() {
        let model = cm("<ol><li>abcd|</li></ol>");
        let (s, e) = model.safe_selection();
        let range = model.state.dom.find_range(s, e);
        assert!(!range.locations[1].position_is_end_of_list_item(2));
    }

    #[test]
    fn location_method_returns_true_for_end_of_list_item() {
        let model = cm("<ol><li>abcd|</li></ol>");
        let (s, e) = model.safe_selection();
        let range = model.state.dom.find_range(s, e);
        assert!(range.locations[1].position_is_end_of_list_item(4));
    }

    #[test]
    fn node_on_border_is_before_or_after_cursor() {
        let range = range_of("<strong>abc</strong>|def");
        let strong_loc = range.locations.first().unwrap();
        assert!(strong_loc.relative_position() == DomLocationPosition::Before);
        let def_loc = range.locations.last().unwrap();
        assert!(def_loc.relative_position() == DomLocationPosition::After);
    }

    #[test]
    fn partially_contained_node_is_inside_of_range() {
        let range = range_of("<strong>abc</strong>{de}|f");
        let def_loc = range.locations.last().unwrap();
        assert!(def_loc.relative_position() == DomLocationPosition::Inside);
    }

    #[test]
    fn cursor_is_inside_all_nodes() {
        let range = range_of("<em><strong>ab|cd</strong></em>");
        range.locations.iter().for_each(|l| {
            assert!(l.relative_position() == DomLocationPosition::Inside)
        })
    }

    #[test]
    fn selection_is_inside_all_nodes() {
        let range = range_of("<em><strong>{ab}|cd</strong></em>");
        range.locations.iter().for_each(|l| {
            assert!(l.relative_position() == DomLocationPosition::Inside)
        })
    }

    #[test]
    fn range_shared_parent() {
        let range = range_of("<em><strong><b>{a</b>b}|</strong>c</em>");
        assert_eq!(range.shared_parent(), DomHandle::from_raw(vec![0, 0]));
    }

    #[test]
    fn range_shared_parent_outside() {
        let range = range_of("<em><strong><b>{a</b>b}|</strong>c</em>");
        assert_eq!(range.shared_parent_outside(), DomHandle::from_raw(vec![0]));
    }

    #[test]
    fn range_shared_parent_flat() {
        let range = range_of("{ab}|");
        assert_eq!(range.shared_parent(), DomHandle::from_raw(vec![]));
    }

    #[test]
    fn range_shared_parent_flat_outside() {
        let range = range_of("{ab}|");
        assert_eq!(range.shared_parent_outside(), DomHandle::from_raw(vec![]));
    }

    #[test]
    fn range_shared_parent_deep_flat() {
        let range = range_of("<em><strong>{ab}|</strong></em>");
        assert_eq!(range.shared_parent(), DomHandle::from_raw(vec![0, 0]));
    }

    #[test]
    fn range_shared_parent_deep_flat_outside() {
        let range = range_of("<em><strong>{ab}|</strong></em>");
        assert_eq!(range.shared_parent_outside(), DomHandle::from_raw(vec![]));
    }

    #[test]
    fn range_find_location() {
        let range = range_of("<em><strong>{ab}|</strong></em>");
        let handle = DomHandle::from_raw(vec![0, 0]);

        let location = range.find_location(&handle).unwrap();

        let expected_kind = DomNodeKind::Formatting(InlineFormatType::Bold);
        assert_eq!(
            *location,
            DomLocation::new(handle, 0, 0, 2, 2, expected_kind)
        );
    }

    #[test]
    fn range_find_location_none() {
        let range = range_of("<em><strong>{ab}|</strong></em>");
        let handle = DomHandle::from_raw(vec![1, 0]);

        let location = range.find_location(&handle);

        assert!(location.is_none());
    }

    fn range_of(model: &str) -> Range {
        let model = cm(model);
        let (s, e) = model.safe_selection();
        model.state.dom.find_range(s, e)
    }
}
