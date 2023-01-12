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

use std::collections::HashMap;
use std::ops::Not;

use widestring::{Utf16Str, Utf16String};

use crate::char::CharExt;
use crate::composer_model::menu_state::MenuStateComputeType;
use crate::dom::nodes::{ContainerNode, LineBreakNode, TextNode, ZwspNode};
use crate::dom::parser::parse;
use crate::dom::to_html::ToHtmlState;
use crate::dom::unicode_string::{UnicodeStr, UnicodeStrExt};
use crate::dom::{Dom, DomLocation};
use crate::{
    ComposerModel, DomHandle, DomNode, Location, ToHtml, UnicodeString,
};

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
    /// assert_eq!(model.state.dom.to_html().to_string(), "aabbcc");
    /// assert_eq!(model.state.start, 2);
    /// assert_eq!(model.state.end, 4);
    /// model.select(Location::from(1), Location::from(5));
    /// assert_eq!(model.to_example_format(), "a{abbc}|c");
    /// ```
    pub fn from_example_format(text: &str) -> Self {
        let text = text.replace('~', &char::zwsp().to_string());

        let mut model = ComposerModel::new();
        model.state.dom = parse(&text).unwrap();

        let mut offset = 0;
        let (start, end, curs) = Self::find_selection_in(
            &model.state.dom,
            model.state.dom.document_node(),
            &mut offset,
        );
        let Some(curs) = curs else {
            panic!("Selection not found");
        };

        fn delete_range(
            model: &mut ComposerModel<Utf16String>,
            loc: &SelectionLocation,
            len: usize,
        ) {
            let mut needs_deletion = false;
            if let DomNode::Text(text_node) =
                model.state.dom.lookup_node_mut(&loc.handle)
            {
                if text_node.data().len() == len {
                    needs_deletion = true;
                } else {
                    text_node.replace_range(
                        Utf16String::new(),
                        loc.offset,
                        loc.offset + len,
                    );
                }
            }
            if needs_deletion {
                model.state.dom.remove(&loc.handle);
            }
        }

        if let (Some(s), Some(e)) = (start, end) {
            if curs.index_in_dom() == e.index_in_dom() + 1 {
                // Cursor after end: foo{bar}|baz
                // The { made an extra codeunit - move the end back 1
                delete_range(&mut model, &e, 2);
                delete_range(&mut model, &s, 1);
                model.state.start = Location::from(s.index_in_dom());
                model.state.end = Location::from(e.index_in_dom() - 1);
            } else if curs.index_in_dom() + 1 == s.index_in_dom() {
                // Cursor before beginning: foo|{bar}baz
                // The |{ made an extra 2 codeunits - move the end back 2
                delete_range(&mut model, &e, 1);
                delete_range(&mut model, &curs, 2);
                model.state.start = Location::from(e.index_in_dom() - 2);
                model.state.end = Location::from(curs.index_in_dom());
            } else {
                panic!(
                    "The cursor ('|') must always be directly before or after \
                    the selection ('{{..}}')! \
                    E.g.: 'foo|{{bar}}baz' or 'foo{{bar}}|baz'."
                );
            }
        } else {
            delete_range(&mut model, &curs, 1);
            model.state.start = Location::from(curs.index_in_dom());
            model.state.end = Location::from(curs.index_in_dom());
        }
        model.compute_menu_state(MenuStateComputeType::KeepIfUnchanged);
        model.state.dom.explicitly_assert_invariants();

        model
    }

    fn find_selection_in(
        model: &Dom<Utf16String>,
        node: &DomNode<Utf16String>,
        offset: &mut usize,
    ) -> (
        Option<SelectionLocation>,
        Option<SelectionLocation>,
        Option<SelectionLocation>,
    ) {
        let mut start = None;
        let mut end = None;
        let mut curs = None;
        match node {
            DomNode::Container(container) => {
                for child in container.children() {
                    let (new_start, new_end, new_curs) =
                        Self::find_selection_in(&model, &child, offset);
                    if start.is_none() {
                        start = new_start;
                    }
                    if end.is_none() {
                        end = new_end;
                    }
                    if curs.is_none() {
                        curs = new_curs;
                    }
                    if model.adds_line_break(&child.handle()) {
                        *offset += 1;
                    }
                    if start.is_none() && end.is_none() && curs.is_some() {
                        break;
                    } else if start.is_some() && end.is_some() {
                        break;
                    }
                }
            }
            DomNode::Text(text_node) => {
                let start_pos = *offset;
                let data: &Utf16Str = text_node.data();
                for ch in data.chars() {
                    if ch == '{' {
                        start = Some(SelectionLocation::new(
                            node.handle(),
                            start_pos,
                            *offset - start_pos,
                        ));
                    } else if ch == '}' {
                        end = Some(SelectionLocation::new(
                            node.handle(),
                            start_pos,
                            *offset - start_pos,
                        ));
                    } else if ch == '|' {
                        curs = Some(SelectionLocation::new(
                            node.handle(),
                            start_pos,
                            *offset - start_pos,
                        ));
                    }
                    *offset += data.char_len(&ch);
                }
            }
            _ => {
                *offset += node.text_len();
            }
        }
        (start, end, curs)
    }

    /// Convert this model to an ASCII-art style representation to be used
    /// for testing and generating examples.
    ///
    /// See [ComposerModel::from_example_format] for the format used.
    pub fn to_example_format(&self) -> String {
        // Clone the model because we will modify it to add selection markers
        let state = &self.state;
        let dom = &state.dom;

        let mut buf = Utf16String::new();

        // Find out which nodes are involved in the selection
        let range = dom.find_range(state.start.into(), state.end.into());

        // Modify the text nodes to add {, } and |
        let selection_start = state.start.into();
        let selection_end = state.end.into();
        let doc_length = dom.document().text_len();
        let root = dom.lookup_node(&dom.document_handle());
        let state = SelectionWritingState::new(
            selection_start,
            selection_end,
            doc_length,
        );
        let locations = range
            .locations
            .iter()
            .map(|l| (l.node_handle.clone(), l.clone()))
            .collect();
        let mut selection_writer = SelectionWriter { state, locations };
        root.fmt_html(
            &mut buf,
            Some(&mut selection_writer),
            ToHtmlState::default(),
        );
        if range.is_empty().not() {
            // we should always have written at least the start of the selection
            // ({ or |) by now.
            assert!(selection_writer.is_selection_written());
        }
        let mut html = buf.to_string();
        if html.is_empty() {
            html = String::from("|");
        }

        // Replace characters with visible ones
        html.replace(char::zwsp(), "~").replace('\u{A0}', "&nbsp;")
    }
}

struct SelectionLocation {
    handle: DomHandle,
    pos: usize,
    offset: usize,
}

impl SelectionLocation {
    fn new(handle: DomHandle, pos: usize, offset: usize) -> Self {
        Self {
            handle,
            pos,
            offset,
        }
    }

    fn index_in_dom(&self) -> usize {
        self.pos + self.offset
    }
}

pub struct SelectionWriter {
    state: SelectionWritingState,
    locations: HashMap<DomHandle, DomLocation>,
}

impl SelectionWriter {
    pub fn write_selection_text_node<S: UnicodeString>(
        &mut self,
        buf: &mut S,
        pos: usize,
        node: &TextNode<S>,
    ) {
        if let Some(loc) = self.locations.get(&node.handle()) {
            let strings_to_add = self.state.advance(loc, node.data().len());
            for (str, i) in strings_to_add.into_iter().rev() {
                buf.insert(pos + i, &S::from(str));
            }
        }
    }

    pub fn write_selection_line_break_node<S: UnicodeString>(
        &mut self,
        buf: &mut S,
        pos: usize,
        node: &LineBreakNode<S>,
    ) {
        if let Some(loc) = self.locations.get(&node.handle()) {
            let strings_to_add = self.state.advance(loc, 1);
            for (str, i) in strings_to_add.into_iter().rev() {
                // Index 1 in line breaks is actually at the end of the '<br />'
                let i = if i == 0 { 0 } else { 6 };
                buf.insert(pos + i, &S::from(str));
            }
        }
    }

    pub fn write_selection_zwsp_node<S: UnicodeString>(
        &mut self,
        buf: &mut S,
        pos: usize,
        node: &ZwspNode<S>,
    ) {
        if let Some(loc) = self.locations.get(&node.handle()) {
            let strings_to_add = self.state.advance(loc, node.data().len());
            for (str, i) in strings_to_add.into_iter().rev() {
                buf.insert(pos + i, &S::from(str));
            }
        }
    }

    pub fn write_selection_block_node<S: UnicodeString>(
        &mut self,
        buf: &mut S,
        pos: usize,
        node: &ContainerNode<S>,
    ) {
        if let Some(loc) = self.locations.get(&node.handle()) {
            if (loc.length > 0 && loc.is_start()) || loc.node_handle.is_root() {
                return;
            }
            let strings_to_add = self.state.advance(loc, 1);
            for (str, _) in strings_to_add.into_iter().rev() {
                buf.insert(pos, &S::from(str));
            }
        }
    }

    pub fn is_selection_written(&self) -> bool {
        self.state.done_first
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
        self.current_pos = location.position + code_units;

        // If we just passed first, write out {
        let mut do_first = !self.done_first && self.first < self.current_pos;

        // If we just passed last or we're at the end, write out }
        let do_last_in_inline = !location.kind.is_block_kind()
            && (self.last <= self.current_pos
                || self.current_pos == self.length);
        let do_last_in_block = location.kind.is_block_kind()
            && !location.node_handle.is_root()
            && self.last < self.current_pos;
        let do_last =
            !self.done_last && (do_last_in_inline || do_last_in_block);

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

#[cfg(all(test, not(target_arch = "wasm32")))]
mod test {
    use speculoos::{prelude::*, AssertionFailure, Spec};
    use widestring::Utf16String;

    use crate::dom::nodes::dom_node::DomNodeKind;
    use crate::dom::{parser, Dom, DomLocation};
    use crate::tests::testutils_composer_model::{cm, restore_whitespace, tx};
    use crate::tests::testutils_conversion::utf16;
    use crate::{
        ComposerModel, ComposerState, DomHandle, DomNode, Location,
        UnicodeString,
    };

    use super::SelectionWritingState;

    #[test]
    fn selection_writing_with_one_character() {
        // We have one text node with one character
        let mut state = SelectionWritingState::new(0, 1, 1);
        let handle = DomHandle::from_raw(vec![0]);
        let location = DomLocation::new(handle, 0, 0, 1, 1, DomNodeKind::Text);

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
        assert_eq!(cm("|").get_content_as_html(), utf16(""));

        assert_eq!(cm("a|").state.start, 1);
        assert_eq!(cm("a|").state.end, 1);
        assert_eq!(cm("a|").get_content_as_html(), utf16("a"));

        assert_eq!(cm("a|b").state.start, 1);
        assert_eq!(cm("a|b").state.end, 1);
        assert_eq!(cm("a|b").get_content_as_html(), utf16("ab"));

        assert_eq!(cm("|ab").state.start, 0);
        assert_eq!(cm("|ab").state.end, 0);
        assert_eq!(cm("|ab").get_content_as_html(), utf16("ab"));

        assert_eq!(cm("foo|").state.start, 3);
        assert_eq!(cm("foo|").state.end, 3);
        assert_eq!(cm("foo|").get_content_as_html(), (utf16("foo")));
    }

    #[test]
    fn cm_creates_correct_component_model_tags() {
        let t0 = cm("AAA<b>B|BB</b>CCC");
        assert_eq!(t0.state.start, 4);
        assert_eq!(t0.state.end, 4);
        assert_eq!(t0.get_content_as_html(), utf16("AAA<b>BBB</b>CCC"));
    }

    #[test]
    fn cm_creates_correct_component_model_newlines() {
        let t0 = cm("|<br />");
        assert_eq!(t0.state.start, 0);
        assert_eq!(t0.state.end, 0);
        assert_eq!(t0.get_content_as_html(), utf16("<br />"));
        // TODO: There should only be one node for the br tag
        //assert_eq!(t0.state.dom.children().len(), 1);

        let t1 = cm("<br />|<br />");
        assert_eq!(t1.state.start, 1);
        assert_eq!(t1.state.end, 1);
        assert_eq!(t1.get_content_as_html(), utf16("<br /><br />"));
        // TODO: assert_eq!(t1.state.dom.children().len(), 2);

        let t2 = cm("<br /><br />|");
        assert_eq!(t2.state.start, 2);
        assert_eq!(t2.state.end, 2);
        assert_eq!(t2.get_content_as_html(), utf16("<br /><br />"));
        // TODO: assert_eq!(t1.state.dom.children().len(), 2);
    }

    #[test]
    fn cm_creates_correct_component_model_multi_code_unit_characters() {
        let t1 = cm("foo|\u{1F4A9}bar");
        assert_eq!(t1.state.start, 3);
        assert_eq!(t1.state.end, 3);
        assert_eq!(t1.get_content_as_html(), utf16("foo\u{1F4A9}bar"));

        let t2 = cm("foo\u{1F4A9}|bar");
        assert_eq!(t2.state.start, 5);
        assert_eq!(t2.state.end, 5);
        assert_eq!(t2.get_content_as_html(), utf16("foo\u{1F4A9}bar"));

        assert_eq!(cm("foo|\u{1F4A9}").state.start, 3);
        assert_eq!(cm("foo|\u{1F4A9}").state.end, 3);
        assert_eq!(
            cm("foo|\u{1F4A9}").get_content_as_html(),
            utf16("foo\u{1F4A9}")
        );

        assert_eq!(cm("foo\u{1F4A9}|").state.start, 5);
        assert_eq!(cm("foo\u{1F4A9}|").state.end, 5);
        assert_eq!(
            cm("foo\u{1F4A9}|").get_content_as_html(),
            utf16("foo\u{1F4A9}")
        );

        assert_eq!(cm("|\u{1F4A9}bar").state.start, 0);
        assert_eq!(cm("|\u{1F4A9}bar").state.end, 0);
        assert_eq!(
            cm("|\u{1F4A9}bar").get_content_as_html(),
            utf16("\u{1F4A9}bar")
        );

        assert_eq!(cm("\u{1F4A9}|bar").state.start, 2);
        assert_eq!(cm("\u{1F4A9}|bar").state.end, 2);
        assert_eq!(
            cm("\u{1F4A9}|bar").get_content_as_html(),
            utf16("\u{1F4A9}bar")
        );
    }

    #[test]
    fn cm_creates_correct_component_model_selection_plain_text() {
        assert_eq!(cm("{a}|").state.start, 0);
        assert_eq!(cm("{a}|").state.end, 1);
        assert_eq!(cm("{a}|").get_content_as_html(), utf16("a"));

        assert_eq!(cm("|{a}").state.start, 1);
        assert_eq!(cm("|{a}").state.end, 0);
        assert_eq!(cm("|{a}").get_content_as_html(), utf16("a"));

        assert_eq!(cm("abc{def}|ghi").state.start, 3);
        assert_eq!(cm("abc{def}|ghi").state.end, 6);
        assert_eq!(
            cm("abc{def}|ghi").get_content_as_html(),
            utf16("abcdefghi")
        );

        assert_eq!(cm("abc|{def}ghi").state.start, 6);
        assert_eq!(cm("abc|{def}ghi").state.end, 3);
        assert_eq!(
            cm("abc|{def}ghi").get_content_as_html(),
            utf16("abcdefghi")
        );
    }

    #[test]
    fn cm_creates_correct_model_selection_multi_code_units_selection() {
        let t3 = cm("\u{1F4A9}{def}|ghi");
        assert_eq!(t3.state.start, 2);
        assert_eq!(t3.state.end, 5);
        assert_eq!(t3.get_content_as_html(), utf16("\u{1F4A9}defghi"));

        let t4 = cm("\u{1F4A9}|{def}ghi");
        assert_eq!(t4.state.start, 5);
        assert_eq!(t4.state.end, 2);
        assert_eq!(t4.get_content_as_html(), utf16("\u{1F4A9}defghi"));

        let t5 = cm("abc{d\u{1F4A9}f}|ghi");
        assert_eq!(t5.state.start, 3);
        assert_eq!(t5.state.end, 7);
        assert_eq!(t5.get_content_as_html(), utf16("abcd\u{1F4A9}fghi"));

        let t6 = cm("abc|{d\u{1F4A9}f}ghi");
        assert_eq!(t6.state.start, 7);
        assert_eq!(t6.state.end, 3);
        assert_eq!(t6.get_content_as_html(), utf16("abcd\u{1F4A9}fghi"));

        let t7 = cm("abc{def}|\u{1F4A9}ghi");
        assert_eq!(t7.state.start, 3);
        assert_eq!(t7.state.end, 6);
        assert_eq!(t7.get_content_as_html(), utf16("abcdef\u{1F4A9}ghi"));

        let t8 = cm("abc|{def}\u{1F4A9}ghi");
        assert_eq!(t8.state.start, 6);
        assert_eq!(t8.state.end, 3);
        assert_eq!(t8.get_content_as_html(), utf16("abcdef\u{1F4A9}ghi"));
    }

    #[test]
    fn cm_parses_selection_spanning_outwards_from_tag_forwards() {
        let model = cm("AAA<b>B{BB</b>C}|CC");
        assert_eq!(model.state.start, 4);
        assert_eq!(model.state.end, 7);
        assert_eq!(model.get_content_as_html(), utf16("AAA<b>BBB</b>CCC"));
    }

    #[test]
    fn cm_parses_selection_spanning_outwards_from_tag_backwards() {
        let model = cm("AAA<b>B|{BB</b>C}CC");
        assert_eq!(model.state.start, 7);
        assert_eq!(model.state.end, 4);
        assert_eq!(model.get_content_as_html(), utf16("AAA<b>BBB</b>CCC"));
    }

    #[test]
    fn tx_formats_selection_spanning_outwards_from_tag_forwards() {
        let model: ComposerModel<Utf16String> =
            ComposerModel::from_state(ComposerState {
                dom: parser::parse("AAA<b>BBB</b>CCC").unwrap(),
                start: Location::from(4),
                end: Location::from(7),
                toggled_format_types: Vec::new(),
            });
        assert_eq!(tx(&model), "AAA<b>B{BB</b>C}|CC");
    }

    #[test]
    fn tx_formats_selection_spanning_outwards_from_tag_backwards() {
        let model: ComposerModel<Utf16String> =
            ComposerModel::from_state(ComposerState {
                dom: parser::parse("AAA<b>BBB</b>CCC").unwrap(),
                start: Location::from(7),
                end: Location::from(4),
                toggled_format_types: Vec::new(),
            });
        assert_eq!(tx(&model), "AAA<b>B|{BB</b>C}CC");
    }

    #[test]
    fn tx_formats_empty_model() {
        let model: ComposerModel<Utf16String> =
            ComposerModel::from_state(ComposerState {
                dom: Dom::default(),
                start: Location::from(1),
                end: Location::from(1),
                toggled_format_types: Vec::new(),
            });
        assert_eq!(tx(&model), "|");
    }

    #[test]
    fn cm_creates_correct_model_selection_multi_code_units_and_tags() {
        let model = cm("a<i>bc|{d<b>ef}\u{1F4A9}g</b>hi</i>");
        assert_eq!(model.state.start, 6);
        assert_eq!(model.state.end, 3);
        assert_eq!(
            model.get_content_as_html(),
            utf16("a<i>bcd<b>ef\u{1F4A9}g</b>hi</i>")
        );
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
        assert_that!("a<br />b|").roundtrips();
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
        assert_that!("aa{<br />}|bb").roundtrips();
        assert_that!("aa|{<br />}bb").roundtrips();
        assert_that!("<ol><li>a|</li></ol>").roundtrips();
    }

    #[test]
    fn selection_across_lists_roundtrips() {
        assert_that!(
            "<ol><li>1{1</li><li>22</li></ol><ol><li>33</li><li>4}|4</li></ol>"
        )
        .roundtrips();
    }

    #[test]
    fn selection_across_lists_with_whitespace_roundtrips() {
        assert_that!(
            "<ol>
                <li>1{1</li>
                <li>22</li>
            </ol>
            <ol>
                <li>33</li>
                <li>4}|4</li>
            </ol>"
        )
        .roundtrips();
    }

    #[test]
    fn selection_ending_at_end_of_list_item_roundtrips() {
        assert_that!(
            "\
            <ul>\
                <li>First item<ul>\
                    <li>{Second item<ul>\
                        <li>Third item</li>\
                        <li>Fourth item}|</li>\
                        <li>Fifth item</li>\
                    </ul></li>\
                </ul></li>\
            </ul>"
        )
        .roundtrips();
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
            let output = restore_whitespace(&tx(&cm(subject)));
            if output != subject {
                AssertionFailure::from_spec(self)
                    .with_expected(String::from(subject))
                    .with_actual(output)
                    .fail();
            }
        }
    }
}
