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

use crate::UnicodeString;

use super::nodes::dom_node::DomNodeKind;
use super::{find_range, Dom, DomLocation, Range};

impl<S> Dom<S>
where
    S: UnicodeString,
{
    pub fn find_extended_range(&self, start: usize, end: usize) -> Range {
        let (s, e) = self.find_extended_selection(start, end);
        find_range::find_range(self, s, e)
    }

    pub fn find_extended_selection(
        &self,
        start: usize,
        end: usize,
    ) -> (usize, usize) {
        let range = find_range::find_range(self, start, end);
        let leaves: Vec<&DomLocation> = range.leaves().collect();
        if leaves.is_empty() {
            return (start, end);
        }

        let first_leaf = leaves.first().unwrap();
        let last_leaf = leaves.last().unwrap();

        (
            self.find_extended_offset_before(start, first_leaf),
            self.find_extended_offset_after(end, last_leaf),
        )
    }

    fn find_extended_offset_before(
        &self,
        start: usize,
        location: &DomLocation,
    ) -> usize {
        if location.kind == DomNodeKind::LineBreak {
            // Item at location is a linebreak, no need to iterate
            return start;
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

        start - offset
    }

    fn find_extended_offset_after(
        &self,
        end: usize,
        location: &DomLocation,
    ) -> usize {
        if location.kind == DomNodeKind::LineBreak {
            // Item at location is a linebreak, no need to iterate
            return end;
        }

        let mut iter = self.iter_from_handle(&location.node_handle);
        // Remove location and use offset instead
        iter.next();
        let mut offset = location.length - location.end_offset;

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

        end + offset
    }
}

#[cfg(test)]
mod test {
    use crate::tests::testutils_composer_model::cm;

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
}
