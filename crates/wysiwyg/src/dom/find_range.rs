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

use crate::dom::nodes::{ContainerNode, DomNode, LineBreakNode, TextNode};
use crate::dom::range::DomLocation;
use crate::dom::unicode_string::UnicodeStrExt;
use crate::dom::{Dom, DomHandle, FindResult, Range};
use crate::UnicodeString;
use std::cmp::{max, min};

pub fn find_range<S>(dom: &Dom<S>, start: usize, end: usize) -> Range
where
    S: UnicodeString,
{
    if dom.children().is_empty() {
        return Range::new(Vec::new());
    }

    // If end < start, we swap start & end to make calculations easier, then
    // reverse the returned ranges
    let is_reversed = end < start;
    let (s, e) = if is_reversed {
        (end, start)
    } else {
        (start, end)
    };

    // TODO: is there really a difference between find_pos and find_range?
    let result = find_pos(dom, &dom.document_handle(), s, e);
    match result {
        FindResult::Found(locations) => {
            let locations: Vec<DomLocation> = if is_reversed {
                locations
                    .iter()
                    .map(|location| location.reversed())
                    .collect()
            } else {
                locations
            };
            Range::new(&locations)
        }
        FindResult::NotFound => Range::new(Vec::new()),
    }
}

/// Find a particular character range in the DOM
pub fn find_pos<S>(
    dom: &Dom<S>,
    node_handle: &DomHandle,
    start: usize,
    end: usize,
) -> FindResult
where
    S: UnicodeString,
{
    let mut offset = 0;
    let locations = do_find_pos(dom, node_handle, start, end, &mut offset);

    if locations.is_empty() {
        FindResult::NotFound
    } else {
        FindResult::Found(locations)
    }
}

fn do_find_pos<S>(
    dom: &Dom<S>,
    node_handle: &DomHandle,
    start: usize,
    end: usize,
    offset: &mut usize,
) -> Vec<DomLocation>
where
    S: UnicodeString,
{
    let node = dom.lookup_node(node_handle);
    let mut locations = Vec::new();
    if *offset > end {
        return locations;
    }
    match node {
        DomNode::Text(n) => {
            if let Some(location) = process_text_node(n, start, end, offset) {
                locations.push(location);
            }
        }
        DomNode::LineBreak(n) => {
            if let Some(location) =
                process_line_break_node(n, start, end, offset)
            {
                locations.push(location);
            }
        }
        DomNode::Container(n) => {
            locations
                .extend(process_container_node(dom, n, start, end, offset));
        }
    }
    locations
}

fn process_container_node<S>(
    dom: &Dom<S>,
    node: &ContainerNode<S>,
    start: usize,
    end: usize,
    offset: &mut usize,
) -> Vec<DomLocation>
where
    S: UnicodeString,
{
    let mut results = Vec::new();
    let container_start = *offset;
    for child in node.children() {
        let child_handle = child.handle();
        assert!(!child_handle.is_root(), "Incorrect child handle!");
        let locations = do_find_pos(dom, &child_handle, start, end, offset);
        if !locations.is_empty() {
            results.extend(locations);
        }
    }
    // If container node is completely selected, include it
    let container_end = *offset;
    let container_node_len = container_end - container_start;
    // We never want to return the root node
    if container_end >= start
        && container_start <= end
        && !node.handle().is_root()
    {
        let start_offset = max(start, container_start) - container_start;
        let end_offset = min(end, container_end) - container_start;
        results.push(DomLocation {
            node_handle: node.handle(),
            position: container_start,
            start_offset,
            end_offset,
            length: container_node_len,
            is_leaf: false,
        })
    }
    results
}

fn process_text_node<S>(
    node: &TextNode<S>,
    start: usize,
    end: usize,
    offset: &mut usize,
) -> Option<DomLocation>
where
    S: UnicodeString,
{
    process_textlike_node(node.handle(), node.data().len(), start, end, offset)
}

fn process_line_break_node<S>(
    node: &LineBreakNode<S>,
    start: usize,
    end: usize,
    offset: &mut usize,
) -> Option<DomLocation>
where
    S: UnicodeString,
{
    // Line breaks are like 1-character text nodes
    process_textlike_node(node.handle(), 1, start, end, offset)
}

fn process_textlike_node(
    handle: DomHandle,
    node_len: usize,
    start: usize,
    end: usize,
    offset: &mut usize,
) -> Option<DomLocation> {
    let node_start = *offset;
    let node_end = node_start + node_len;

    // Increase offset to keep track of the current position
    *offset += node_len;

    let outside_selection_range = start > node_end || end < node_start;
    let is_cursor = start == end;

    // Intersect selection and node ranges with a couple of exceptions
    if outside_selection_range
        // Selection start is at the end of a node, but it's not a cursor
        || (start == node_end && !is_cursor)
        // Selection end is at the start of a node, but not on position 0
        || (node_start == end && end != 0)
    {
        // Node discarded, it's not selected
        None
    } else {
        // Diff between selected position and the start position of the node
        let start_offset = max(start, node_start) - node_start;
        let end_offset = min(end, node_end) - node_start;

        Some(DomLocation {
            node_handle: handle,
            position: node_start,
            start_offset,
            end_offset,
            length: node_len,
            is_leaf: true,
        })
    }
}

#[cfg(test)]
mod test {
    // TODO: more tests for start and end of ranges

    use widestring::Utf16String;

    use super::*;
    use crate::tests::testutils_composer_model::{cm, restore_whitespace_u16};
    use crate::tests::testutils_conversion::utf16;
    use crate::tests::testutils_dom::{b, dom, tn};
    use crate::ToHtml;

    fn found_single_node(
        handle: DomHandle,
        position: usize,
        start_offset: usize,
        end_offset: usize,
        length: usize,
    ) -> FindResult {
        FindResult::Found(vec![DomLocation {
            node_handle: handle,
            position,
            start_offset,
            end_offset,
            length,
            is_leaf: true,
        }])
    }

    fn ranges_to_html(
        dom: &Dom<Utf16String>,
        range: &Range,
    ) -> Vec<Utf16String> {
        range
            .locations
            .iter()
            .map(|location| {
                restore_whitespace_u16(
                    &dom.lookup_node(&location.node_handle).to_html(),
                )
            })
            .collect()
    }

    #[test]
    fn finding_a_node_within_an_empty_dom_returns_not_found() {
        let d = dom(&[]);
        assert_eq!(
            find_pos(&d, &d.document_handle(), 0, 0),
            FindResult::NotFound
        );
    }

    #[test]
    fn finding_a_node_within_a_single_text_node_is_found() {
        let d = dom(&[tn("foo")]);
        assert_eq!(
            find_pos(&d, &d.document_handle(), 1, 1),
            found_single_node(DomHandle::from_raw(vec![0]), 0, 1, 1, 3)
        );
    }

    #[test]
    fn finding_a_node_within_flat_text_nodes_is_found() {
        let d = dom(&[tn("foo"), tn("bar")]);
        assert_eq!(
            find_pos(&d, &d.document_handle(), 0, 0),
            found_single_node(DomHandle::from_raw(vec![0]), 0, 0, 0, 3)
        );
        assert_eq!(
            find_pos(&d, &d.document_handle(), 1, 1),
            found_single_node(DomHandle::from_raw(vec![0]), 0, 1, 1, 3)
        );
        assert_eq!(
            find_pos(&d, &d.document_handle(), 2, 2),
            found_single_node(DomHandle::from_raw(vec![0]), 0, 2, 2, 3)
        );
        assert_eq!(
            find_pos(&d, &d.document_handle(), 3, 3),
            found_single_node(DomHandle::from_raw(vec![0]), 0, 3, 3, 3)
        );
        assert_eq!(
            find_pos(&d, &d.document_handle(), 3, 4),
            found_single_node(DomHandle::from_raw(vec![1]), 3, 0, 1, 3)
        );
        // TODO: break up this test and name parts!
        assert_eq!(
            find_pos(&d, &d.document_handle(), 4, 4),
            found_single_node(DomHandle::from_raw(vec![1]), 3, 1, 1, 3)
        );
        assert_eq!(
            find_pos(&d, &d.document_handle(), 4, 4),
            found_single_node(DomHandle::from_raw(vec![1]), 3, 1, 1, 3)
        );
        assert_eq!(
            find_pos(&d, &d.document_handle(), 5, 5),
            found_single_node(DomHandle::from_raw(vec![1]), 3, 2, 2, 3)
        );
        assert_eq!(
            find_pos(&d, &d.document_handle(), 5, 5),
            found_single_node(DomHandle::from_raw(vec![1]), 3, 2, 2, 3)
        );
        assert_eq!(
            find_pos(&d, &d.document_handle(), 6, 6),
            found_single_node(DomHandle::from_raw(vec![1]), 3, 3, 3, 3)
        );
    }

    // TODO: comprehensive test like above for non-flat nodes

    #[test]
    fn finding_a_range_within_an_empty_dom_returns_no_nodes() {
        let d = dom(&[]);
        let range = d.find_range(0, 0);
        assert_eq!(range, Range::new(Vec::new()));
    }

    #[test]
    fn finding_a_range_within_the_single_text_node_works() {
        let d = dom(&[tn("foo bar baz")]);
        let range = d.find_range(4, 7);

        let leaves: Vec<&DomLocation> = range.leaves().collect();
        assert_eq!(leaves.len(), 1);

        let loc = leaves[0];
        assert_eq!(loc.start_offset, 4);
        assert_eq!(loc.end_offset, 7);

        if let DomNode::Text(t) = d.lookup_node(&loc.node_handle) {
            assert_eq!(*t.data(), utf16("foo bar baz"));
        } else {
            panic!("Should have been a text node!")
        }

        assert_eq!(loc.node_handle.raw(), &vec![0]);
    }

    #[test]
    fn finding_a_range_that_includes_the_end_works_simple_case() {
        let d = dom(&[tn("foo bar baz")]);
        let range = d.find_range(4, 11);

        let leaves: Vec<&DomLocation> = range.leaves().collect();
        assert_eq!(leaves.len(), 1);

        let loc = leaves[0];
        assert_eq!(loc.start_offset, 4);
        assert_eq!(loc.end_offset, 11);

        if let DomNode::Text(t) = d.lookup_node(&loc.node_handle) {
            assert_eq!(*t.data(), utf16("foo bar baz"));
        } else {
            panic!("Should have been a text node!")
        }

        assert_eq!(loc.node_handle.raw(), &vec![0]);
    }

    #[test]
    fn finding_a_range_within_some_nested_node_works() {
        let d = dom(&[tn("foo "), b(&[tn("bar")]), tn(" baz")]);
        let range = d.find_range(5, 6);

        let leaves: Vec<&DomLocation> = range.leaves().collect();
        assert_eq!(leaves.len(), 1);

        let loc = leaves[0];
        assert_eq!(loc.start_offset, 1);
        assert_eq!(loc.end_offset, 2);

        if let DomNode::Text(t) = d.lookup_node(&loc.node_handle) {
            assert_eq!(*t.data(), utf16("bar"));
        } else {
            panic!("Should have been a text node!")
        }

        assert_eq!(loc.node_handle.raw(), &vec![1, 0]);
    }

    #[test]
    fn finding_a_range_across_several_nodes_works() {
        let d = cm("test<b>ing a </b>new feature|").state.dom;
        let range = d.find_range(2, 12);

        // 3 text nodes + bold node
        assert_eq!(4, range.locations.len());
        let html_of_ranges = ranges_to_html(&d, &range);
        assert_eq!(utf16("test"), html_of_ranges[0]);
        assert_eq!(utf16("ing a "), html_of_ranges[1]);
        assert_eq!(utf16("<b>ing a </b>"), html_of_ranges[2]);
        assert_eq!(utf16("new feature"), html_of_ranges[3]);
    }

    #[test]
    fn finding_a_range_across_several_nested_nodes_works() {
        let d = cm("test<b>ing <i>a </i></b>new feature|").state.dom;
        let range = d.find_range(2, 12);
        // 4 text nodes + bold node + italic node
        assert_eq!(6, range.locations.len());
        let html_of_ranges = ranges_to_html(&d, &range);
        assert_eq!(utf16("test"), html_of_ranges[0]);
        assert_eq!(utf16("ing "), html_of_ranges[1]);
        assert_eq!(utf16("a "), html_of_ranges[2]);
        assert_eq!(utf16("<i>a </i>"), html_of_ranges[3]);
        assert_eq!(utf16("<b>ing <i>a </i></b>"), html_of_ranges[4]);
        assert_eq!(utf16("new feature"), html_of_ranges[5]);
    }

    #[test]
    fn finding_a_range_inside_several_nested_nodes_returns_text_node() {
        let d = cm("test<b>ing <i>a </i></b>new feature|").state.dom;
        let range = d.find_range(9, 10);
        // Selected the 'a' character inside the <i> tag, but as it only
        // covers it partially, only the text node is selected
        assert_eq!(
            range,
            Range {
                locations: vec![
                    DomLocation {
                        node_handle: DomHandle::from_raw(vec![1, 1, 0]),
                        start_offset: 1,
                        end_offset: 2,
                        position: 8,
                        length: 2,
                        is_leaf: true
                    },
                    DomLocation {
                        node_handle: DomHandle::from_raw(vec![1, 1]),
                        start_offset: 1,
                        end_offset: 2,
                        position: 8,
                        length: 2,
                        is_leaf: false
                    },
                    DomLocation {
                        node_handle: DomHandle::from_raw(vec![1]),
                        start_offset: 5,
                        end_offset: 6,
                        position: 4,
                        length: 6,
                        is_leaf: false
                    }
                ]
            }
        );
    }

    #[test]
    fn finding_a_range_spanning_nested_nodes_selects_text_node_and_parent() {
        let d = cm("test<b>ing <i>a </i></b>new feature|").state.dom;
        // The range of the whole <i> tag
        let range = d.find_range(8, 11);
        // 2 text nodes + italic node
        assert_eq!(4, range.locations.len());
        let html_of_ranges = ranges_to_html(&d, &range);
        assert_eq!(utf16("a "), html_of_ranges[0]);
        assert_eq!(utf16("<i>a </i>"), html_of_ranges[1]);
        assert_eq!(utf16("<b>ing <i>a </i></b>"), html_of_ranges[2]);
        assert_eq!(utf16("new feature"), html_of_ranges[3]);
    }
}
