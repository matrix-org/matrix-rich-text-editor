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

use crate::{DomHandle, UnicodeString};

use super::nodes::dom_node::DomNodeKind;
use super::{find_range, Dom, DomLocation, Range};

impl<S> Dom<S>
where
    S: UnicodeString,
{
    /// Returns a range from given start and end, extended up to the next
    /// structure node, list item or linebreak on each side.
    pub fn find_extended_range(&self, start: usize, end: usize) -> Range {
        let (s, e) = self.find_extended_selection(start, end);
        find_range::find_range(self, s, e)
    }

    /// Returns a selection from given start and end, extended up to the next
    /// structure node, list item or linebreak on each side.
    pub fn find_extended_selection(
        &self,
        start: usize,
        end: usize,
    ) -> (usize, usize) {
        let range = find_range::find_range(self, start, end);
        let leaves: Vec<&DomLocation> = range
            .leaves()
            .filter(|loc| loc.position < range.end())
            .collect();
        if leaves.is_empty() {
            return (start, end);
        }

        let first_leaf = leaves.first().unwrap();
        let last_leaf = leaves.last().unwrap();
        let extended_start = start - self.extended_offset_before(first_leaf);
        let extended_end = end + self.extended_offset_after(last_leaf);

        (extended_start, extended_end)
    }

    /// Returns the offset before a location until a structure
    /// node, list item or linebreak is found.
    fn extended_offset_before(&self, location: &DomLocation) -> usize {
        if location.kind == DomNodeKind::LineBreak {
            // Item at location is a linebreak, no need to iterate
            return location.start_offset;
        }

        let mut iter = self.iter_from_handle(&location.node_handle).rev();
        // Remove location and use offset instead
        iter.next();
        let mut offset = location.start_offset;

        for node in iter {
            if node.is_text_node() {
                offset += node.text_len();
            } else if node.is_line_break() {
                offset += node.text_len();
                break;
            } else if node.is_block_node() || node.is_list_item() {
                break;
            }
        }

        offset
    }

    /// Returns the offset after a location until a structure
    /// node, list item or linebreak is found.
    fn extended_offset_after(&self, location: &DomLocation) -> usize {
        if location.kind == DomNodeKind::LineBreak
            || self.is_last_child_of_list(&location.node_handle)
        {
            // Item at location is a linebreak or the last child of a list, no need to iterate
            return location.length - location.end_offset;
        }

        let mut iter = self.iter_from_handle(&location.node_handle);
        // Remove location and use offset instead
        iter.next();
        let mut offset = location.length - location.end_offset;

        for node in iter {
            if node.is_text_node() {
                offset += node.text_len();
                if self.is_last_child_of_list(&node.handle()) {
                    break;
                }
            } else if node.is_line_break() {
                offset += node.text_len();
                break;
            } else if node.is_block_node() || node.is_list_item() {
                break;
            }
        }

        offset
    }

    /// Returns if given node is the last child of a list.
    /// This works recursively which means that being the last child
    /// should be considered at its current depth. e.g. the last list
    /// item and the last text node within it would both return true.
    fn is_last_child_of_list(&self, handle: &DomHandle) -> bool {
        if !handle.has_parent() {
            return false;
        }
        let index_in_parent = handle.index_in_parent();
        let parent = self.parent(handle);
        if parent.children().len() - 1 == index_in_parent {
            if parent.is_list() {
                true
            } else {
                self.is_last_child_of_list(&handle.parent_handle())
            }
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{tests::testutils_composer_model::cm, DomHandle};

    #[test]
    fn find_extended_selection_retrieves_single_text_node() {
        let dom = cm("abcdef|").state.dom;
        assert_eq!(dom.find_extended_selection(3, 3), (0, 6));
    }

    #[test]
    fn find_extended_selection_stops_at_leading_trailing_line_breaks() {
        let dom = cm("abc<br />def<br />ghi|").state.dom;
        assert_eq!(dom.find_extended_selection(5, 6), (3, 8));
    }

    #[test]
    fn find_extended_selection_stops_at_list_location() {
        let dom = cm("<ol><li>~abc</li><li>~def</li></ol>~ghi<br />jkl|")
            .state
            .dom;
        assert_eq!(dom.find_extended_selection(9, 10), (8, 13));
    }

    #[test]
    fn find_extended_selection_dont_stop_on_format_tags() {
        let dom = cm("<strong>abc</strong>defg<em>hij</em>kl|").state.dom;
        assert_eq!(dom.find_extended_selection(4, 6), (0, 12));
    }

    #[test]
    fn find_extended_selection_stops_immediately_on_selected_linebreaks() {
        let dom = cm("abc<br />def<br />ghi|").state.dom;
        assert_eq!(dom.find_extended_selection(3, 8), (3, 8));
        assert_eq!(dom.find_extended_selection(4, 6), (3, 8));
    }

    #[test]
    fn find_extended_selection_stops_inside_list() {
        let dom = cm("abc<ol><li>~def|</li></ol>").state.dom;
        assert_eq!(dom.find_extended_selection(4, 4), (3, 7));
    }

    #[test]
    fn find_extended_selection_on_list_border_stops() {
        let dom = cm("abc<ol><li>~def</li></ol>~ghi|").state.dom;
        assert_eq!(dom.find_extended_selection(3, 3), (0, 3));
        assert_eq!(dom.find_extended_selection(7, 7), (3, 7));
        assert_eq!(dom.find_extended_selection(8, 8), (7, 11));
    }

    #[test]
    fn find_extended_selection_from_last_list_item_stops_end_of_list() {
        let dom =
            cm("<ol><li>~abc</li><li><strong>~de</strong>f</li></ol>~ghi|")
                .state
                .dom;
        assert_eq!(dom.find_extended_selection(6, 6), (4, 8));
    }

    #[test]
    fn test_is_last_child_of_list() {
        let dom =
            cm("abc<ol><li>~def</li><li>~g<strong>hi</strong></li></ol>~jkl|")
                .state
                .dom;
        // "abc" is not the last child of a list
        assert!(!dom.is_last_child_of_list(&DomHandle::from_raw(vec![0])));
        // The actual list is not the last child of a list
        assert!(!dom.is_last_child_of_list(&DomHandle::from_raw(vec![1])));
        // The first list item is not the last child of a list
        assert!(!dom.is_last_child_of_list(&DomHandle::from_raw(vec![1, 0])));
        // The second list item is the last child of a list
        assert!(dom.is_last_child_of_list(&DomHandle::from_raw(vec![1, 1])));
        // "~g" is not the last child of a list
        assert!(!dom.is_last_child_of_list(&DomHandle::from_raw(vec![1, 1, 0])));
        // The strong node is the last child of a list
        assert!(dom.is_last_child_of_list(&DomHandle::from_raw(vec![1, 1, 1])));
        // "hi" is the last child of a list
        assert!(
            dom.is_last_child_of_list(&DomHandle::from_raw(vec![1, 1, 1, 0]))
        );
        // "~jkl" is not the last child of a list
        assert!(!dom.is_last_child_of_list(&DomHandle::from_raw(vec![2])));
    }
}
