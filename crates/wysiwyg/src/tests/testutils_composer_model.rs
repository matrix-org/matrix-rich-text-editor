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

#![cfg(test)]

//! Convenience functions to allow working with ComposerModel instances based
//! on ASCII-art-style representions.
//!
//! We provide 2 functions: [cm] to create a ComposerModel from text, and
//! [tx] to convert a ComposerModel to text.
//!
//! ## Format
//!
//! The text format is HTML with 3 special characters: `{` and `}` to indicate
//! the current selections, and `|` to indicate the cursor position.
//!
//! For example, `aa{bb}|cc` means the text `aabbcc`, with a selection starting
//! at 2 and ending at 4.
//!
//! The `|` is mandatory, but you can leave out both `{` and `}` to indicate
//! that no characters are selected, so `aa|bb` means the text `aabb`, with a
//! selection starting at 2 and ending at 2.
//!
//! If `{` and `}` are included, the `|` must be immediately before `{`, or
//! immediately after `|`. So `aa|{bb}cc` means the text `aabbcc`, with a
//! selection starting at 4 and ending at 2 (i.e. the user dragged from right
//! to left to select).
//!
//! The characters `{`, `}` or `|` must not appear anywhere else in the text.
//!
//! HTML works, so `AA<b>B|B</b>CC` means a text node containing `AA`, followed
//! but a bold node containing a text node containing `BB`, followed by a text
//! node containing `CC`, with a selection starting and ending at 3.
//!
//! ## Examples
//!
//! See the test immediately below this comment in the source code. Doctests
//! can't be used inside #[cfg(test)], so it is included as a normal test.

#[test]
fn example() {
    let mut model = cm("aa{bb}|cc");
    assert_eq!(model.state.dom.to_html().to_utf8(), "aabbcc");
    assert_eq!(model.state.start, 2);
    assert_eq!(model.state.end, 4);

    model.select(Location::from(1), Location::from(5));
    assert_eq!(tx(&model), "a{abbc}|c");
}

use widestring::Utf16String;

use crate::dom::nodes::{DomNode, TextNode};
use crate::dom::parser::parse;
use crate::dom::{Dom, DomLocation, MultipleNodesRange, Range, SameNodeRange};
use crate::{ComposerModel, ComposerState, Location, ToHtml, UnicodeString};

/// Create a ComposerModel from a text representation. See the [testutils]
/// module documentation for details about the format.
pub fn cm(text: &str) -> ComposerModel<Utf16String> {
    let text_u16 = Utf16String::from_str(text).into_vec();

    /// Return the UTF-16 code unit for a character
    /// Panics if s is more than one code unit long.
    fn utf16_code_unit(s: &str) -> u16 {
        let mut ret = Utf16String::new();
        ret.push_str(s);
        assert_eq!(ret.len(), 1);
        ret.into_vec()[0]
    }

    fn find(haystack: &[u16], needle: &str) -> Option<usize> {
        let mut skip_count = 0; // How many tag characters we have seen
        let mut in_tag = false; // Are we in a tag now?

        let needle = utf16_code_unit(needle);
        let open = utf16_code_unit("<");
        let close = utf16_code_unit(">");

        for (i, &ch) in haystack.iter().enumerate() {
            if ch == needle {
                return Some(i - skip_count);
            } else if ch == open {
                in_tag = true;
            } else if ch == close {
                skip_count += 1;
                in_tag = false;
            }
            if in_tag {
                skip_count += 1;
            }
        }
        None
    }

    let curs = find(&text_u16, "|").expect(&format!(
        "ComposerModel text did not contain a '|' symbol: '{}'",
        String::from_utf16(&text_u16)
            .expect("ComposerModel text was not UTF-16"),
    ));

    let s = find(&text_u16, "{");
    let e = find(&text_u16, "}");

    let mut model = ComposerModel {
        state: ComposerState::new(),
        previous_states: Vec::new(),
        next_states: Vec::new(),
    };
    model.state.dom = parse(&text).unwrap();

    fn delete_range(
        model: &mut ComposerModel<Utf16String>,
        p1: usize,
        p2: usize,
    ) {
        model.do_replace_text_in(Utf16String::new(), p1, p2);
    }

    if let (Some(s), Some(e)) = (s, e) {
        if curs == e + 1 {
            // Cursor after end: foo{bar}|baz
            // The { made an extra codeunit - move the end back 1
            delete_range(&mut model, s, s + 1);
            delete_range(&mut model, e - 1, e + 1);
            model.state.start = Location::from(s);
            model.state.end = Location::from(e - 1);
        } else if curs == s - 1 {
            // Cursor before beginning: foo|{bar}baz
            // The |{ made an extra 2 codeunits - move the end back 2
            delete_range(&mut model, s - 1, s + 1);
            delete_range(&mut model, e - 2, e - 1);
            model.state.start = Location::from(e - 2);
            model.state.end = Location::from(curs);
        } else {
            panic!(
                "The cursor ('|') must always be directly before or after \
                    the selection ('{{..}}')! \
                    E.g.: 'foo|{{bar}}baz' or 'foo{{bar}}|baz'."
            );
        }
    } else {
        delete_range(&mut model, curs, curs + 1);
        model.state.start = Location::from(curs);
        model.state.end = Location::from(curs);
    }

    model
}

/// Convert a ComposerModel into a text representation. See the [testutils]
/// module documentation for details about the format.
pub fn tx(model: &ComposerModel<Utf16String>) -> String {
    // TODO: break into a separate file
    fn update_text_node_with_cursor(
        text_node: &mut TextNode<Utf16String>,
        range: SameNodeRange,
    ) {
        let orig_s: usize = range.start_offset.into();
        let orig_e: usize = range.end_offset.into();
        let (s, e) = if orig_s < orig_e {
            (orig_s, orig_e)
        } else {
            (orig_e, orig_s)
        };

        let data = text_node.data();
        let mut new_data;
        if s == e {
            new_data = data[..s].to_string();
            new_data.push('|');
            new_data += data[s..].to_string().as_str();
        } else {
            new_data = data[..s].to_string();
            if orig_s < orig_e {
                new_data.push('{');
            } else {
                new_data += "|{";
            }
            new_data += data[s..e].to_string().as_str();
            if orig_s < orig_e {
                new_data += "}|";
            } else {
                new_data.push('}');
            }
            new_data += data[e..].to_string().as_str();
        }
        text_node.set_data(Utf16String::from_str(&new_data));
    }

    fn update_text_node(
        node: &mut TextNode<Utf16String>,
        offset: usize,
        s: &'static str,
    ) {
        let data = node.data();
        let mut new_start_data = data[..offset].to_string();
        new_start_data.push_str(s);
        new_start_data += &data[offset..].to_string();
        node.set_data(Utf16String::from_str(&new_start_data));
    }

    struct SelectionWritingState {
        // Counts how far through the whole document we have got (code units)
        current_pos: usize,

        // Have we written out the "{" or "|{" yet?
        done_first: bool,

        // Have we written out the "}" or "}|" yet?
        done_last: bool,

        // The length of the whole document
        length: usize,

        // The location of the leftmost part of the selection (code_units)
        first: usize,

        // The location of the rightmost part of the selection (code_units)
        last: usize,

        // Does the selection start at the right and end at the left?
        reversed: bool,
    }

    impl SelectionWritingState {
        fn new(start: usize, end: usize, length: usize) -> Self {
            let reversed = start > end;

            let (first, last): (usize, usize) = if start > end {
                (end, start)
            } else {
                (start, end)
            };

            Self {
                current_pos: 0,
                done_first: false,
                done_last: false,
                length,
                first,
                last,
                reversed,
            }
        }

        /// Move forward code_units, and return what markers we should add
        /// to the current node.
        ///
        /// Returns a Vec of (marker, offset) pairs. Each marker should be
        /// added within its node at the supplied offset.
        fn advance(
            &mut self,
            location: &DomLocation,
            code_units: usize,
        ) -> Vec<(&'static str, usize)> {
            self.current_pos += code_units;

            // If we just passed first, write out {
            let do_first = !self.done_first && self.first < self.current_pos;

            // If we just passed last or we're at the end, write out }
            let do_last = !self.done_last
                && (self.last < self.current_pos
                    || self.current_pos == self.length);

            // Remember that we have passed them, so we don't repeat
            self.done_first = self.done_first || do_first;
            self.done_last = self.done_last || do_last;

            let mut ret = Vec::new();

            // Add the markers we want to write
            if do_first {
                ret.push((
                    self.first_marker(),
                    if self.reversed {
                        location.end_offset
                    } else {
                        location.start_offset
                    },
                ));
            };

            if do_last {
                ret.push((
                    self.last_marker(),
                    if self.reversed {
                        location.start_offset
                    } else {
                        location.end_offset
                    },
                ));
            };

            // Return a list of markers to write and their locations
            ret
        }

        /// Return the marker to insert into the leftmost edge of the selection
        fn first_marker(&self) -> &'static str {
            if self.reversed {
                "|{"
            } else {
                "{"
            }
        }

        /// Return the marker to insert into the rightmost edge of the selection
        fn last_marker(&self) -> &'static str {
            if self.reversed {
                "}"
            } else {
                "}|"
            }
        }
    }

    /// Insert {, } and | to mark the start and end of a range
    /// start is the absolute position of the start of the range
    /// end is the absolute position of the end of the range
    fn write_selection_multi(
        dom: &mut Dom<Utf16String>,
        range: MultipleNodesRange,
        start: usize,
        end: usize,
    ) {
        let mut state =
            SelectionWritingState::new(start, end, dom.document().text_len());

        for location in range.locations {
            let mut node = dom.lookup_node_mut(location.node_handle.clone());
            match &mut node {
                DomNode::Container(_) => {}
                DomNode::Text(n) => {
                    let strings_to_add =
                        state.advance(&location, n.data().len());
                    for (s, offset) in strings_to_add {
                        update_text_node(n, offset, s);
                    }
                }
            }
        }

        // Since we are in a multi-node selection, we should always write {
        assert!(state.done_first);
    }

    // Clone the model because we will modify it to add selection markers
    let state = model.state.clone();
    let mut dom = state.dom;

    // Find out which nodes are involved in the selection
    let range = dom.find_range(state.start.into(), state.end.into());

    // Modify the text nodes to a {, } and |
    match range {
        Range::SameNode(range) => {
            let mut node = dom.lookup_node_mut(range.node_handle.clone());
            match &mut node {
                DomNode::Container(_) => {
                    panic!("Don't know how to tx in a non-text node")
                }
                DomNode::Text(n) => update_text_node_with_cursor(n, range),
            };
        }
        Range::NoNode => panic!("No node!"),
        Range::MultipleNodes(range) => write_selection_multi(
            &mut dom,
            range,
            state.start.into(),
            state.end.into(),
        ),
    }

    dom.to_string()
}

mod test {
    use speculoos::{prelude::*, AssertionFailure, Spec};

    use crate::tests::testutils_conversion::utf16;

    use super::*;

    #[test]
    fn cm_creates_correct_component_model() {
        // TODO: can we split and/or make these tests clearer?
        assert_eq!(cm("|").state.start, 0);
        assert_eq!(cm("|").state.end, 0);
        assert_eq!(cm("|").get_html(), utf16(""));

        assert_eq!(cm("a|").state.start, 1);
        assert_eq!(cm("a|").state.end, 1);
        assert_eq!(cm("a|").get_html(), utf16("a"));

        assert_eq!(cm("a|b").state.start, 1);
        assert_eq!(cm("a|b").state.end, 1);
        assert_eq!(cm("a|b").get_html(), utf16("ab"));

        assert_eq!(cm("|ab").state.start, 0);
        assert_eq!(cm("|ab").state.end, 0);
        assert_eq!(cm("|ab").get_html(), utf16("ab"));

        assert_eq!(cm("foo|").state.start, 3);
        assert_eq!(cm("foo|").state.end, 3);
        assert_eq!(cm("foo|").get_html(), (utf16("foo")));

        let t0 = cm("AAA<b>B|BB</b>CCC");
        assert_eq!(t0.state.start, 4);
        assert_eq!(t0.state.end, 4);
        assert_eq!(t0.get_html(), utf16("AAA<b>BBB</b>CCC"));

        let t1 = cm("foo|\u{1F4A9}bar");
        assert_eq!(t1.state.start, 3);
        assert_eq!(t1.state.end, 3);
        assert_eq!(t1.get_html(), utf16("foo\u{1F4A9}bar"));

        let t2 = cm("foo\u{1F4A9}|bar");
        assert_eq!(t2.state.start, 5);
        assert_eq!(t2.state.end, 5);
        assert_eq!(t2.get_html(), utf16("foo\u{1F4A9}bar"));

        assert_eq!(cm("foo|\u{1F4A9}").state.start, 3);
        assert_eq!(cm("foo|\u{1F4A9}").state.end, 3);
        assert_eq!(cm("foo|\u{1F4A9}").get_html(), utf16("foo\u{1F4A9}"));

        assert_eq!(cm("foo\u{1F4A9}|").state.start, 5);
        assert_eq!(cm("foo\u{1F4A9}|").state.end, 5);
        assert_eq!(cm("foo\u{1F4A9}|").get_html(), utf16("foo\u{1F4A9}"));

        assert_eq!(cm("|\u{1F4A9}bar").state.start, 0);
        assert_eq!(cm("|\u{1F4A9}bar").state.end, 0);
        assert_eq!(cm("|\u{1F4A9}bar").get_html(), utf16("\u{1F4A9}bar"));

        assert_eq!(cm("\u{1F4A9}|bar").state.start, 2);
        assert_eq!(cm("\u{1F4A9}|bar").state.end, 2);
        assert_eq!(cm("\u{1F4A9}|bar").get_html(), utf16("\u{1F4A9}bar"));

        assert_eq!(cm("{a}|").state.start, 0);
        assert_eq!(cm("{a}|").state.end, 1);
        assert_eq!(cm("{a}|").get_html(), utf16("a"));

        assert_eq!(cm("|{a}").state.start, 1);
        assert_eq!(cm("|{a}").state.end, 0);
        assert_eq!(cm("|{a}").get_html(), utf16("a"));

        assert_eq!(cm("abc{def}|ghi").state.start, 3);
        assert_eq!(cm("abc{def}|ghi").state.end, 6);
        assert_eq!(cm("abc{def}|ghi").get_html(), utf16("abcdefghi"));

        assert_eq!(cm("abc|{def}ghi").state.start, 6);
        assert_eq!(cm("abc|{def}ghi").state.end, 3);
        assert_eq!(cm("abc|{def}ghi").get_html(), utf16("abcdefghi"));

        let t3 = cm("\u{1F4A9}{def}|ghi");
        assert_eq!(t3.state.start, 2);
        assert_eq!(t3.state.end, 5);
        assert_eq!(t3.get_html(), utf16("\u{1F4A9}defghi"));

        let t4 = cm("\u{1F4A9}|{def}ghi");
        assert_eq!(t4.state.start, 5);
        assert_eq!(t4.state.end, 2);
        assert_eq!(t4.get_html(), utf16("\u{1F4A9}defghi"));

        let t5 = cm("abc{d\u{1F4A9}f}|ghi");
        assert_eq!(t5.state.start, 3);
        assert_eq!(t5.state.end, 7);
        assert_eq!(t5.get_html(), utf16("abcd\u{1F4A9}fghi"));

        let t6 = cm("abc|{d\u{1F4A9}f}ghi");
        assert_eq!(t6.state.start, 7);
        assert_eq!(t6.state.end, 3);
        assert_eq!(t6.get_html(), utf16("abcd\u{1F4A9}fghi"));

        let t7 = cm("abc{def}|\u{1F4A9}ghi");
        assert_eq!(t7.state.start, 3);
        assert_eq!(t7.state.end, 6);
        assert_eq!(t7.get_html(), utf16("abcdef\u{1F4A9}ghi"));

        let t8 = cm("abc|{def}\u{1F4A9}ghi");
        assert_eq!(t8.state.start, 6);
        assert_eq!(t8.state.end, 3);
        assert_eq!(t8.get_html(), utf16("abcdef\u{1F4A9}ghi"));
    }

    #[test]
    fn cm_and_tx_roundtrip() {
        assert_that!("|").roundtrips();
        assert_that!("a|").roundtrips();
        assert_that!("a|b").roundtrips();
        assert_that!("|ab").roundtrips();
        assert_that!("foo|\u{1F4A9}bar").roundtrips();
        assert_that!("foo\u{1F4A9}|bar").roundtrips();
        assert_that!("foo|\u{1F4A9}").roundtrips();
        assert_that!("foo\u{1F4A9}|").roundtrips();
        assert_that!("|\u{1F4A9}bar").roundtrips();
        assert_that!("\u{1F4A9}|bar").roundtrips();
        assert_that!("{a}|").roundtrips();
        assert_that!("|{a}").roundtrips();
        assert_that!("abc{def}|ghi").roundtrips();
        assert_that!("abc|{def}ghi").roundtrips();
        assert_that!("\u{1F4A9}{def}|ghi").roundtrips();
        assert_that!("\u{1F4A9}|{def}ghi").roundtrips();
        assert_that!("abc{d\u{1F4A9}f}|ghi").roundtrips();
        assert_that!("abc|{d\u{1F4A9}f}ghi").roundtrips();
        assert_that!("abc{def}|\u{1F4A9}ghi").roundtrips();
        assert_that!("abc|{def}\u{1F4A9}ghi").roundtrips();
        assert_that!("AAA<b>B|BB</b>CCC").roundtrips();
        assert_that!("AAA<b>{BBB}|</b>CCC").roundtrips();
        assert_that!("AAA<b>|{BBB}</b>CCC").roundtrips();
        assert_that!("AAA<b>B<em>B|</em>B</b>CCC").roundtrips();
        assert_that!("AAA<b>B|<em>B</em>B</b>CCC").roundtrips();
        assert_that!("AAA<b>B<em>{B}|</em>B</b>CCC").roundtrips();
        assert_that!("AAA<b>B<em>|{B}</em>B</b>CCC").roundtrips();
        assert_that!("AAA<b>B<em>B</em>B</b>C|CC").roundtrips();
        assert_that!("AAA<b>B<em>B</em>B</b>{C}|CC").roundtrips();
        assert_that!("AAA<b>B<em>B</em>B</b>|{C}CC").roundtrips();
        assert_that!("AA{A<b>B<em>B</em>B</b>C}|CC").roundtrips();
        assert_that!("AA|{A<b>B<em>B</em>B</b>C}CC").roundtrips();
        assert_that!("{AAA<b>B<em>B</em>B</b>CCC}|").roundtrips();
        assert_that!("|{AAA<b>B<em>B</em>B</b>CCC}").roundtrips();
    }

    trait Roundtrips<T> {
        fn roundtrips(&self);
    }

    impl<'s, T> Roundtrips<T> for Spec<'s, T>
    where
        T: AsRef<str>,
    {
        fn roundtrips(&self) {
            let subject = self.subject.as_ref();
            let output = tx(&cm(subject));
            if output != subject {
                AssertionFailure::from_spec(self)
                    .with_expected(String::from(subject))
                    .with_actual(output)
                    .fail();
            }
        }
    }
}
