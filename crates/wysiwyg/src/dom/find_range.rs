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

use crate::dom::nodes::{ContainerNode, DomNode};
use crate::dom::{Dom, DomHandle, FindResult, Range};

use super::SameNodeRange;

pub fn find_range<C>(dom: &Dom<C>, start: usize, end: usize) -> Range
where
    C: Clone,
{
    if dom.children().is_empty() {
        return Range::NoNode;
    }

    // TODO: We walk the whole tree twice (by calling find_pos twice) -
    // maybe we can do better than that?  (But very unlikely to be a
    // performance problem.)

    // TODO: more tests that directly exercise this beginning and end stuff
    let (find_start, find_end) = match start.cmp(&end) {
        std::cmp::Ordering::Equal => {
            // When there is no range, only a cursor, we use "end" style,
            // staying within a tag if we are near the end
            let pos =
                find_pos(dom, dom.document_handle(), end, RangeLocation::End);
            (pos.clone(), pos)
        }
        std::cmp::Ordering::Less => {
            // Start and end are in expected order - use normal start
            // and end style for find them.
            (
                find_pos(
                    dom,
                    dom.document_handle(),
                    start,
                    RangeLocation::Start,
                ),
                find_pos(dom, dom.document_handle(), end, RangeLocation::End),
            )
        }
        std::cmp::Ordering::Greater => {
            // Start and end are in opposite order - use opposite start
            // and end style for find them.
            (
                find_pos(dom, dom.document_handle(), start, RangeLocation::End),
                find_pos(dom, dom.document_handle(), end, RangeLocation::Start),
            )
        }
    };

    // TODO: needs careful handling when on the boundary of 2 ranges:
    // we want to be greedy about when we state something is the same range
    // - maybe find_pos should return 2 nodes when we are on the boundary?
    match (find_start, find_end) {
        (
            FindResult::Found {
                node_handle: start_handle,
                offset: start_offset,
            },
            FindResult::Found {
                node_handle: end_handle,
                offset: end_offset,
            },
        ) => {
            if start_handle == end_handle {
                Range::SameNode(SameNodeRange {
                    node_handle: start_handle,
                    start_offset,
                    end_offset,
                })
            } else {
                Range::TooDifficultForMe
            }
        }
        _ => Range::TooDifficultForMe,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum RangeLocation {
    Start,
    End,
}

/// Find a particular character position in the DOM
///
/// location controls whether we are looking for the start or the end
/// of a range. When we are on the border of a tag, if we are looking for
/// the start, we return the character at the beginning of the next tag,
/// whereas if we are looking for the end of a range, we return the
/// position after the last character of the previous tag.
///
/// When searching for an individual character (rather than a range), you
/// should ask for RangeLocation::End.
fn find_pos<C>(
    dom: &Dom<C>,
    node_handle: DomHandle,
    offset: usize,
    location: RangeLocation,
) -> FindResult
where
    C: Clone,
{
    // TODO: consider whether cloning DomHandles is damaging performance,
    // and look for ways to pass around references, maybe.

    let node = dom.lookup_node(node_handle.clone());
    match node {
        DomNode::Text(n) => {
            let len = n.data().len();
            if (location == RangeLocation::Start && offset < len)
                || (location == RangeLocation::End && offset <= len)
            {
                FindResult::Found {
                    node_handle,
                    offset,
                }
            } else {
                FindResult::NotFound {
                    new_offset: offset - len,
                }
            }
        }
        DomNode::Container(n) => {
            process_container_node(dom, n, offset, location)
        }
    }
}

fn process_container_node<C: Clone>(
    dom: &Dom<C>,
    node: &ContainerNode<C>,
    offset: usize,
    location: RangeLocation,
) -> FindResult {
    let mut off = offset;
    for child in node.children() {
        let child_handle = child.handle();
        assert!(!child_handle.is_root(), "Incorrect child handle!");
        let find_child = find_pos(dom, child_handle, off, location);
        match find_child {
            FindResult::Found { .. } => {
                return find_child;
            }
            FindResult::NotFound { new_offset } => {
                off = new_offset;
            }
        }
    }
    FindResult::NotFound { new_offset: off }
}

#[cfg(test)]
mod test {
    // TODO: more tests for start and end of ranges

    use super::*;

    use crate::tests::testutils_dom::{b, dom, tn};
    use crate::ToHtml;

    use crate::dom::Dom;

    #[test]
    fn finding_a_node_within_an_empty_dom_returns_not_found() {
        let d: Dom<u16> = dom(&[]);
        assert_eq!(
            find_pos(&d, d.document_handle(), 0, RangeLocation::Start),
            FindResult::NotFound { new_offset: 0 }
        );
    }

    #[test]
    fn finding_a_node_within_a_single_text_node_is_found() {
        let d: Dom<u16> = dom(&[tn("foo")]);
        assert_eq!(
            find_pos(&d, d.document_handle(), 1, RangeLocation::Start),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![0]),
                offset: 1
            }
        );
    }

    #[test]
    fn finding_a_node_within_flat_text_nodes_is_found() {
        let d: Dom<u16> = dom(&[tn("foo"), tn("bar")]);
        assert_eq!(
            find_pos(&d, d.document_handle(), 0, RangeLocation::Start),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![0]),
                offset: 0
            }
        );
        assert_eq!(
            find_pos(&d, d.document_handle(), 1, RangeLocation::Start),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![0]),
                offset: 1
            }
        );
        assert_eq!(
            find_pos(&d, d.document_handle(), 2, RangeLocation::Start),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![0]),
                offset: 2
            }
        );
        assert_eq!(
            find_pos(&d, d.document_handle(), 3, RangeLocation::Start),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![1]),
                offset: 0
            }
        );
        assert_eq!(
            find_pos(&d, d.document_handle(), 3, RangeLocation::End),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![0]),
                offset: 3
            }
        );
        // TODO: break up this test and name parts!
        assert_eq!(
            find_pos(&d, d.document_handle(), 4, RangeLocation::Start),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![1]),
                offset: 1
            }
        );
        assert_eq!(
            find_pos(&d, d.document_handle(), 4, RangeLocation::End),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![1]),
                offset: 1
            }
        );
        assert_eq!(
            find_pos(&d, d.document_handle(), 5, RangeLocation::Start),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![1]),
                offset: 2
            }
        );
        assert_eq!(
            find_pos(&d, d.document_handle(), 5, RangeLocation::End),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![1]),
                offset: 2
            }
        );
        assert_eq!(
            find_pos(&d, d.document_handle(), 6, RangeLocation::End),
            FindResult::Found {
                node_handle: DomHandle::from_raw(vec![1]),
                offset: 3
            }
        );
    }

    // TODO: comprehensive test like above for non-flat nodes

    #[test]
    fn finding_a_range_within_an_empty_dom_returns_no_node() {
        let d: Dom<u16> = dom(&[]);
        let range = d.find_range(0, 0);
        assert_eq!(range, Range::NoNode);
    }

    #[test]
    fn finding_a_range_within_the_single_text_node_works() {
        let d = dom(&[tn("foo bar baz")]);
        let range = d.find_range(4, 7);

        if let Range::SameNode(range) = range {
            assert_eq!(range.start_offset, 4);
            assert_eq!(range.end_offset, 7);

            if let DomNode::Text(t) = d.lookup_node(range.node_handle.clone()) {
                assert_eq!(t.data(), "foo bar baz".to_html());
            } else {
                panic!("Should have been a text node!")
            }

            assert_eq!(range.node_handle.raw(), &vec![0]);
        } else {
            panic!("Should have been a SameNodeRange: {:?}", range)
        }
    }

    #[test]
    fn finding_a_range_that_includes_the_end_works_simple_case() {
        let d = dom(&[tn("foo bar baz")]);
        let range = d.find_range(4, 11);

        if let Range::SameNode(range) = range {
            assert_eq!(range.start_offset, 4);
            assert_eq!(range.end_offset, 11);

            if let DomNode::Text(t) = d.lookup_node(range.node_handle.clone()) {
                assert_eq!(t.data(), "foo bar baz".to_html());
            } else {
                panic!("Should have been a text node!")
            }

            assert_eq!(range.node_handle.raw(), &vec![0]);
        } else {
            panic!("Should have been a SameNodeRange: {:?}", range)
        }
    }

    #[test]
    fn finding_a_range_within_some_nested_node_works() {
        let d = dom(&[tn("foo "), b(&[tn("bar")]), tn(" baz")]);
        let range = d.find_range(5, 6);

        if let Range::SameNode(range) = range {
            assert_eq!(range.start_offset, 1);
            assert_eq!(range.end_offset, 2);

            if let DomNode::Text(t) = d.lookup_node(range.node_handle.clone()) {
                assert_eq!(t.data(), "bar".to_html());
            } else {
                panic!("Should have been a text node!")
            }

            assert_eq!(range.node_handle.raw(), &vec![1, 0]);
        } else {
            panic!("Should have been a SameNodeRange: {:?}", range)
        }
    }
}
