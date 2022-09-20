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

use std::collections::HashSet;
use widestring::Utf16String;

use crate::dom::nodes::TextNode;
use crate::dom::parser::parse;
use crate::dom::{Dom, DomLocation, MultipleNodesRange, Range};
use crate::{ComposerModel, ComposerState, DomNode, Location};

impl ComposerModel<Utf16String> {
    /// Convenience function to allow working with ComposerModel instances
    /// based on ASCII-art-style representions.
    ///
    /// We provide 2 functions: [ComposerModel::from_example_format] to create#
    /// a ComposerModel from text, and [ComposerModel::to_example_format] to
    /// convert a ComposerModel to text.
    ///
    /// ## Format
    ///
    /// The text format is HTML with 3 special characters: `{` and `}` to
    /// indicate the current selections, and `|` to indicate the cursor
    /// position.
    ///
    /// For example, `aa{bb}|cc` means the text `aabbcc`, with a selection
    /// starting at 2 and ending at 4.
    ///
    /// The `|` is mandatory, but you can leave out both `{` and `}` to
    /// indicate that no characters are selected, so `aa|bb` means the text
    /// `aabb`, with a selection starting at 2 and ending at 2.
    ///
    /// If `{` and `}` are included, the `|` must be immediately before `{`, or
    /// immediately after `|`. So `aa|{bb}cc` means the text `aabbcc`, with a
    /// selection starting at 4 and ending at 2 (i.e. the user dragged from
    /// right to left to select).
    ///
    /// The characters `{`, `}` or `|` must not appear anywhere else in the
    /// text.
    ///
    /// Any occurrence of the `~` character is replaced with the Unicode
    /// code point U+200B ZERO WIDTH SPACE inside from_example_format.
    /// Similarly, when converting back using [to_example_format], any
    /// ZERO WIDTH SPACE is replaced by `~`. This allows test cases to use
    /// zero-width spaces without being very confusing. (Zero-width spaces
    /// are used in various places in the model to allow the selection cursor
    /// to be positioned e.g. inside an empty tag.)
    ///
    /// HTML works, so `AA<b>B|B</b>CC` means a text node containing `AA`,
    /// followed by a bold node containing a text node containing `BB`,
    /// followed by a text node containing `CC`, with a selection starting and
    /// ending at 3.
    ///
    /// ## Examples
    ///
    /// ```
    /// use wysiwyg::{ComposerModel, Location, ToHtml, UnicodeString};
    ///
    /// let mut model = ComposerModel::from_example_format("aa{bb}|cc");
    /// assert_eq!(model.state.dom.to_html().to_utf8(), "aabbcc");
    /// assert_eq!(model.state.start, 2);
    /// assert_eq!(model.state.end, 4);
    /// model.select(Location::from(1), Location::from(5));
    /// assert_eq!(model.to_example_format(), "a{abbc}|c");
    /// ```
    pub fn from_example_format(text: &str) -> Self {
        let text = text.replace("~", "\u{200b}");
        let text_u16 = Utf16String::from_str(&text).into_vec();

        let curs = find_char(&text_u16, "|").expect(&format!(
            "ComposerModel text did not contain a '|' symbol: '{}'",
            String::from_utf16(&text_u16)
                .expect("ComposerModel text was not UTF-16"),
        ));

        let s = find_char(&text_u16, "{");
        let e = find_char(&text_u16, "}");

        let mut model = ComposerModel {
            state: ComposerState::new(),
            previous_states: Vec::new(),
            next_states: Vec::new(),
            reversed_actions: HashSet::new(),
            disabled_actions: HashSet::new(),
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
        model.compute_menu_state();

        model
    }

    /// Convert this model to an ASCII-art style representation to be used
    /// for testing and generating examples.
    ///
    /// See [ComposerModel::from_example_format] for the format used.
    pub fn to_example_format(&self) -> String {
        // Clone the model because we will modify it to add selection markers
        let state = self.state.clone();
        let mut dom = state.dom;

        // Find out which nodes are involved in the selection
        let range = dom.find_range(state.start.into(), state.end.into());

        // Modify the text nodes to add {, } and |
        match range {
            Range::SameNode(range) => {
                let mrange = dom.convert_same_node_range_to_multi(range);
                write_selection_multi(
                    &mut dom,
                    mrange,
                    state.start.into(),
                    state.end.into(),
                )
            }
            Range::NoNode => (), // No selection, so insert no text
            Range::MultipleNodes(range) => write_selection_multi(
                &mut dom,
                range,
                state.start.into(),
                state.end.into(),
            ),
        }

        dom.to_string().replace("\u{200b}", "~")
    }
}

/// Return the UTF-16 code unit for a character
/// Panics if s is more than one code unit long.
fn utf16_code_unit(s: &str) -> u16 {
    let mut ret = Utf16String::new();
    ret.push_str(s);
    assert_eq!(ret.len(), 1);
    ret.into_vec()[0]
}

/// Find a single utf16 code unit (needle) in haystack
fn find_char(haystack: &[u16], needle: &str) -> Option<usize> {
    let mut skip_count = 0; // How many tag characters we have seen
    let mut in_tag = false; // Are we in a tag now?

    // Track the contents of the tag we are inside, so we know whether we've
    // seen a br tag.
    let mut tag_contents: Vec<u16> = Vec::new();

    let needle = utf16_code_unit(needle);
    let open = utf16_code_unit("<");
    let close = utf16_code_unit(">");
    let space = utf16_code_unit(" ");
    let forward_slash = utf16_code_unit("/");

    let br_tag = Utf16String::from_str("br").into_vec();

    for (i, &ch) in haystack.iter().enumerate() {
        if ch == needle {
            return Some(i - skip_count);
        } else if ch == open {
            in_tag = true;
        } else if ch == close {
            // Skip this character (>), unless we've found a br tag, in which
            // case the whole tag will be worth 1 code unit, so we don't
            // increase skip count.
            if tag_contents != br_tag {
                skip_count += 1;
            }
            in_tag = false;
            tag_contents.clear();
        }
        if in_tag {
            skip_count += 1;

            // Track what's inside this tag, but ignore spaces and slashes
            if !(ch == open || ch == space || ch == forward_slash) {
                tag_contents.push(ch);
            }
        }
    }
    None
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

#[derive(Debug)]
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
    /// added within its node at the supplied offset. These markers are
    /// returned in order of where they should be inserted, so may be
    /// inserted in reverse order to avoid invalidating other handles and
    /// offsets.
    fn advance(
        &mut self,
        location: &DomLocation,
        code_units: usize,
    ) -> Vec<(&'static str, usize)> {
        if self.current_pos == 0 {
            // If this is the first location we have visited, update our start
            // position to the start of this location.
            self.current_pos = location.position;
        }
        self.current_pos += code_units;

        // If we just passed first, write out {
        let mut do_first = !self.done_first && self.first < self.current_pos;

        // If we just passed last or we're at the end, write out }
        let do_last = !self.done_last
            && (self.last <= self.current_pos
                || self.current_pos == self.length);

        // In some weird circumstances with empty text nodes, we might
        // do_last when we haven't done_first, so make sure we do_first too.
        if do_last && !self.done_first {
            do_first = true
        }

        // Remember that we have passed them, so we don't repeat
        self.done_first = self.done_first || do_first;
        self.done_last = self.done_last || do_last;

        let mut ret = Vec::new();

        // Add the markers we want to write
        if do_first && do_last && location.start_offset == location.end_offset {
            ret.push(("|", location.start_offset));
        } else {
            if do_first {
                ret.push((
                    self.first_marker(),
                    if self.reversed {
                        location.end_offset
                    } else {
                        location.start_offset
                    },
                ));
            }

            if do_last {
                ret.push((
                    self.last_marker(),
                    if self.reversed {
                        location.start_offset
                    } else {
                        location.end_offset
                    },
                ));
            }
        }

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

    let mut nodes_to_add = Vec::new();
    for location in range.locations {
        let handle = &location.node_handle;
        let mut node = dom.lookup_node_mut(handle);
        match &mut node {
            DomNode::Container(_) => {}
            DomNode::LineBreak(_) => {
                let strings_to_add = state.advance(&location, 1);
                for (s, offset) in strings_to_add.iter().rev() {
                    nodes_to_add.push((
                        handle.clone(),
                        handle.index_in_parent() + offset,
                        DomNode::new_text(Utf16String::from_str(s)),
                    ));
                }
            }
            DomNode::Text(n) => {
                let strings_to_add = state.advance(&location, n.data().len());
                for (s, offset) in strings_to_add.iter().rev() {
                    update_text_node(n, *offset, s);
                }
            }
        }
    }
    if !nodes_to_add.is_empty() {
        for (handle, idx, node) in nodes_to_add.into_iter().rev() {
            let parent = dom.lookup_node_mut(&handle.parent_handle());
            if let DomNode::Container(parent) = parent {
                parent.insert_child(idx, node);
            } else {
                panic!("Parent node was not a container!");
            }
        }
    }

    // Since we are in a multi-node selection, we should always write {
    assert!(state.done_first);
}

#[cfg(test)]
mod test {
    use speculoos::{prelude::*, AssertionFailure, Spec};
    use std::collections::HashSet;
    use widestring::Utf16String;

    use crate::dom::{parser, Dom, DomLocation};
    use crate::tests::testutils_composer_model::{cm, tx};
    use crate::tests::testutils_conversion::utf16;
    use crate::{ComposerModel, ComposerState, DomHandle, DomNode, Location};

    use super::SelectionWritingState;

    #[test]
    fn selection_writing_with_one_character() {
        // We have one text node with one character
        let mut state = SelectionWritingState::new(0, 1, 1);
        let handle = DomHandle::from_raw(vec![0]);
        let location = DomLocation::new(handle, 0, 0, 1, 1, true);

        // When we advance
        let strings_to_add = state.advance(&location, 1);

        // The character should be selected
        assert_eq!(strings_to_add, vec![("{", 0), ("}|", 1),]);
    }

    // These tests use cm and tx for brevity, but those call directly through
    // to the code above.

    #[test]
    fn cm_creates_correct_component_model_plain() {
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
    }

    #[test]
    fn cm_creates_correct_component_model_tags() {
        let t0 = cm("AAA<b>B|BB</b>CCC");
        assert_eq!(t0.state.start, 4);
        assert_eq!(t0.state.end, 4);
        assert_eq!(t0.get_html(), utf16("AAA<b>BBB</b>CCC"));
    }

    #[test]
    fn cm_creates_correct_component_model_newlines() {
        let t0 = cm("|<br />");
        assert_eq!(t0.state.start, 0);
        assert_eq!(t0.state.end, 0);
        assert_eq!(t0.get_html(), utf16("<br />"));
        // TODO: There should only be one node for the br tag
        //assert_eq!(t0.state.dom.children().len(), 1);

        let t1 = cm("<br />|<br />");
        assert_eq!(t1.state.start, 1);
        assert_eq!(t1.state.end, 1);
        assert_eq!(t1.get_html(), utf16("<br /><br />"));
        // TODO: assert_eq!(t1.state.dom.children().len(), 2);

        let t2 = cm("<br /><br />|");
        assert_eq!(t2.state.start, 2);
        assert_eq!(t2.state.end, 2);
        assert_eq!(t2.get_html(), utf16("<br /><br />"));
        // TODO: assert_eq!(t1.state.dom.children().len(), 2);
    }

    #[test]
    fn cm_creates_correct_component_model_multi_code_unit_characters() {
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
    }

    #[test]
    fn cm_creates_correct_component_model_selection_plain_text() {
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
    }

    #[test]
    fn cm_creates_correct_model_selection_multi_code_units_selection() {
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
    fn cm_parses_selection_spanning_outwards_from_tag_forwards() {
        let model = cm("AAA<b>B{BB</b>C}|CC");
        assert_eq!(model.state.start, 4);
        assert_eq!(model.state.end, 7);
        assert_eq!(model.get_html(), utf16("AAA<b>BBB</b>CCC"));
    }

    #[test]
    fn cm_parses_selection_spanning_outwards_from_tag_backwards() {
        let model = cm("AAA<b>B|{BB</b>C}CC");
        assert_eq!(model.state.start, 7);
        assert_eq!(model.state.end, 4);
        assert_eq!(model.get_html(), utf16("AAA<b>BBB</b>CCC"));
    }

    #[test]
    fn tx_formats_selection_spanning_outwards_from_tag_forwards() {
        let model: ComposerModel<Utf16String> = ComposerModel {
            state: ComposerState {
                dom: parser::parse("AAA<b>BBB</b>CCC").unwrap(),
                start: Location::from(4),
                end: Location::from(7),
            },
            previous_states: Vec::new(),
            next_states: Vec::new(),
            reversed_actions: HashSet::new(),
            disabled_actions: HashSet::new(),
        };
        assert_eq!(tx(&model), "AAA<b>B{BB</b>C}|CC");
    }

    #[test]
    fn tx_formats_selection_spanning_outwards_from_tag_backwards() {
        let model: ComposerModel<Utf16String> = ComposerModel {
            state: ComposerState {
                dom: parser::parse("AAA<b>BBB</b>CCC").unwrap(),
                start: Location::from(7),
                end: Location::from(4),
            },
            previous_states: Vec::new(),
            next_states: Vec::new(),
            reversed_actions: HashSet::new(),
            disabled_actions: HashSet::new(),
        };
        assert_eq!(tx(&model), "AAA<b>B|{BB</b>C}CC");
    }

    #[test]
    fn tx_formats_empty_model() {
        let model: ComposerModel<Utf16String> = ComposerModel {
            state: ComposerState {
                dom: Dom::new(Vec::new()),
                start: Location::from(1),
                end: Location::from(1),
            },
            previous_states: Vec::new(),
            next_states: Vec::new(),
            reversed_actions: HashSet::new(),
            disabled_actions: HashSet::new(),
        };
        assert_eq!(tx(&model), "");
    }

    #[test]
    fn cm_creates_correct_model_selection_multi_code_units_and_tags() {
        let model = cm("a<i>bc|{d<b>ef}\u{1F4A9}g</b>hi</i>");
        assert_eq!(model.state.start, 6);
        assert_eq!(model.state.end, 3);
        assert_eq!(model.get_html(), utf16("a<i>bcd<b>ef\u{1F4A9}g</b>hi</i>"));
    }

    #[test]
    fn cm_converts_tilda_to_zero_width_space() {
        let model = cm("~|");
        assert_eq!(model.state.start, 1);
        assert_eq!(model.state.end, 1);

        if let DomNode::Text(node) = &model.state.dom.document().children()[0] {
            assert_eq!(node.data(), "\u{200b}");
        } else {
            panic!("Expected a text node!");
        }
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
        assert_that!("A{AA<b>B}|BB</b>CCC").roundtrips();
        assert_that!("A|{AA<b>B}BB</b>CCC").roundtrips();
        assert_that!("AAA<b>B{BB</b>C}|CC").roundtrips();
        assert_that!("AAA<b>B|{BB</b>C}CC").roundtrips();
        assert_that!("<ul><li>~|</li></ul>").roundtrips();
        assert_that!("<br />|").roundtrips();
        assert_that!("<br /><br />|").roundtrips();
        assert_that!("<br />|<br />").roundtrips();
        assert_that!("<br />|<br />").roundtrips();
        assert_that!("a<br />|<br />b").roundtrips();
        assert_that!("a<br />b|<br />c").roundtrips();
        assert_that!("a<br />|b<br />c").roundtrips();
        assert_that!("<b>a<br />|b<br />c</b>").roundtrips();
        assert_that!("|<br />").roundtrips();
        assert_that!("aaa<br />|bbb").roundtrips();
        assert_that!("aaa|<br />bbb").roundtrips();
        assert_that!("aa{a<br />b}|bb").roundtrips();
        assert_that!("aa|{a<br />b}bb").roundtrips();
        assert_that!("aa{<br />b}|bb").roundtrips();
        assert_that!("aa|{<br />b}bb").roundtrips();
        assert_that!("aa{a<br />b}|bb").roundtrips();
        assert_that!("aa|{a<br />}bb").roundtrips();
        // TODO: easier after refactor assert_that!("aa{<br />}|bb").rou
        // TODO: assert_that!("aa|{<br />}bb").roundtrips();
        assert_that!("<ol><li>|</li></ol>").roundtrips();
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
