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

use crate::composer_model::delete_text::Direction;
use crate::composer_model::example_format::SelectionWriter;
use crate::dom::dom_handle::DomHandle;
use crate::dom::to_html::{ToHtml, ToHtmlState};
use crate::dom::to_markdown::{MarkdownError, MarkdownOptions, ToMarkdown};
use crate::dom::to_plain_text::ToPlainText;
use crate::dom::to_raw_text::ToRawText;
use crate::dom::to_tree::ToTree;
use crate::dom::unicode_string::{UnicodeStr, UnicodeStrExt, UnicodeStringExt};
use crate::dom::UnicodeString;
use html_escape;
use std::ops::Range;

// categories of character for backspace/delete word
#[derive(PartialEq, Eq, Debug)]
pub enum CharType {
    Whitespace,
    Punctuation,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextNode<S>
where
    S: UnicodeString,
{
    data: S,
    handle: DomHandle,
}

impl<S> TextNode<S>
where
    S: UnicodeString,
{
    /// Create a new TextNode
    ///
    /// NOTE: Its handle() will be unset until you call set_handle() or
    /// append() it to another node.
    pub fn from(data: S) -> Self {
        Self {
            data,
            handle: DomHandle::new_unset(),
        }
    }

    pub fn data(&self) -> &S::Str {
        &self.data
    }

    /// Extends the text before and after given range, until reaching a whitespace or the
    /// boundaries of the `UnicodeString`.
    /// Returns the extended data, as well as the length of the extension before and after
    /// the given range, with the current character encoding.
    pub(crate) fn extended_text_for_range(
        &self,
        range: Range<usize>,
    ) -> (&S::Str, usize, usize) {
        let offset_before = self.data.previous_whitespace_offset(range.start);
        let offset_after = self.data.next_whitespace_offset(range.end);
        let new_start = range.start - offset_before;
        let new_end = range.end + offset_after;

        (&self.data[new_start..new_end], offset_before, offset_after)
    }

    pub fn set_data(&mut self, data: S) {
        self.data = data;
    }

    pub fn replace_range(&mut self, data: S, start: usize, end: usize) {
        let mut new_data = self.data[..start].to_owned();
        new_data.push(data);
        new_data.push(self.data[end..].to_owned());
        self.data = new_data;
    }

    pub fn handle(&self) -> DomHandle {
        self.handle.clone()
    }

    pub fn set_handle(&mut self, handle: DomHandle) {
        self.handle = handle;
    }

    /// Returns true if the text_node contains only blank characters
    pub fn is_blank(&self) -> bool {
        self.data.chars().all(|c| c.is_whitespace())
    }

    /// Returns true if the text_node contains only blank characters
    /// in the specified range
    pub fn is_blank_in_range(&self, range: Range<usize>) -> bool {
        self.data[range].chars().all(|c| c.is_whitespace())
    }

    pub fn remove_trailing_line_break(&mut self) -> bool {
        if self.data.chars().last() == Some('\n') {
            self.data.pop_last();
            true
        } else {
            false
        }
    }

    pub fn clone_with_range(&self, range: Range<usize>) -> TextNode<S> {
        TextNode::from(self.data[range].to_owned())
    }

    /// This gets the character at the cursor offset, considering the
    /// direction of travel
    fn char_at_offset(
        &self,
        offset: usize,
        direction: &Direction,
    ) -> Option<char> {
        self.data()
            .chars()
            .nth(direction.get_index_from_cursor(offset))
    }

    /// This gets the character type at the cursor offset, considering the
    /// direction of travel
    pub fn char_type_at_offset(
        &self,
        offset: usize,
        direction: &Direction,
    ) -> Option<CharType> {
        let char = self.char_at_offset(offset, direction);
        char.map(get_char_type)
    }

    /// When moving through a node, the cursor counts as inside the node
    /// at one end, but not the other. This function determines that.
    pub fn offset_is_inside_node(
        &self,
        current_offset: usize,
        direction: &Direction,
    ) -> bool {
        let node_length = self.data().len();
        match direction {
            Direction::Forwards => current_offset < node_length,
            Direction::Backwards => current_offset > 0,
        }
    }

    /// Required due to zero length text node existence
    pub fn is_empty(&self) -> bool {
        self.data().len() != 0
    }

    /// Push content of the given text node into self.
    pub(crate) fn push(&mut self, other_node: &TextNode<S>) {
        let mut text_data = self.data().to_owned();
        let other_text_data = other_node.data();
        if !other_text_data.is_empty() {
            text_data.push(other_text_data.to_owned());
        }
        self.set_data(text_data);
    }

    /// Slice this text node after given position.
    /// Returns a new text node of the same kind with the
    /// removed content.
    pub fn slice_after(&mut self, position: usize) -> TextNode<S> {
        assert!(position <= self.data.len());
        let data_after = self.data[position..].to_owned();
        self.set_data(self.data[..position].to_owned());
        TextNode::from(data_after)
    }

    /// Slice this text node before given position.
    /// Returns a new text node of the same kind with the
    /// removed content.
    pub fn slice_before(&mut self, position: usize) -> TextNode<S> {
        assert!(position <= self.data.len());
        let data_before = self.data[..position].to_owned();
        self.set_data(self.data[position..].to_owned());
        TextNode::from(data_before)
    }
}

/// Given a character, determine its type
fn get_char_type(c: char) -> CharType {
    // in order to determine where a ctrl/opt + delete type operation finishes
    // we need to distinguish between whitespace (nb no newline characters), punctuation
    // and then everything else is treated as the same type
    if c.is_whitespace() {
        CharType::Whitespace
    } else if c.is_ascii_punctuation() {
        CharType::Punctuation
    } else {
        CharType::Other
    }
}

impl<S> ToHtml<S> for TextNode<S>
where
    S: UnicodeString,
{
    fn fmt_html(
        &self,
        buf: &mut S,
        selection_writer: Option<&mut SelectionWriter>,
        state: ToHtmlState,
        _as_message: bool,
    ) {
        let cur_pos = buf.len();
        let string = self.data.to_string();

        let mut escaped = html_escape::encode_text(&string).to_string();
        // Replace all pairs of spaces with non-breaking ones. Transforms
        // `a     b` to `a\u{A0}\u{A0}\u{A0}\u{A0} b`, which will render
        // exactly as five spaces like in the input.
        if !state.is_inside_code_block {
            escaped = escaped.replace("  ", "\u{A0}\u{A0}");

            if state.is_last_node_in_parent
                && escaped.chars().next_back().map_or(false, |c| c == ' ')
            {
                // If this is the last node and it ends in a space, replace that
                // space with a non-breaking one.
                escaped.replace_range(escaped.len() - 1.., "\u{A0}");
            }

            if state.is_first_node_in_parent
                && escaped.chars().next().map_or(false, |c| c == ' ')
            {
                // If this is the first node and it starts with a space, replace that
                // space with a non-breaking one.
                escaped.replace_range(..1, "\u{A0}");
            }
        }
        buf.push(escaped.as_str());

        if let Some(selection_writer) = selection_writer {
            selection_writer.write_selection_text_node(buf, cur_pos, self);
        }
    }
}

impl<S> ToRawText<S> for TextNode<S>
where
    S: UnicodeString,
{
    fn to_raw_text(&self) -> S {
        self.data.clone()
    }
}

impl<S> ToPlainText<S> for TextNode<S>
where
    S: UnicodeString,
{
    fn to_plain_text(&self) -> S {
        self.data.clone()
    }
}

impl<S> ToTree<S> for TextNode<S>
where
    S: UnicodeString,
{
    fn to_tree_display(&self, continuous_positions: Vec<usize>) -> S {
        let mut description = S::from("\"");
        description.push(self.data.clone());
        description.push('"');
        return self.tree_line(
            description,
            self.handle.raw().len(),
            continuous_positions,
        );
    }
}

impl<S> ToMarkdown<S> for TextNode<S>
where
    S: UnicodeString,
{
    fn fmt_markdown(
        &self,
        buffer: &mut S,
        options: &MarkdownOptions,
        _as_message: bool,
    ) -> Result<(), MarkdownError<S>> {
        if options.contains(MarkdownOptions::NO_ESCAPE) {
            buffer.push(self.data());
            return Ok(());
        }

        let mut escaped = S::default();

        for c in self.data().chars() {
            match c {
                // https://spec.commonmark.org/0.30/#ascii-punctuation-character
                '!' | '"' | '#' | '$' | '%' | '&' | '\'' | '(' | ')' | '*'
                | '+' | ',' | '-' | '.' | '/' | ':' | ';' | '<' | '=' | '>'
                | '?' | '@' | '[' | '\\' | ']' | '^' | '_' | '`' | '{'
                | '|' | '}' | '~' => {
                    escaped.push('\\');
                    escaped.push(c);
                }
                _ => escaped.push(c),
            }
        }

        buffer.push(escaped);

        Ok(())
    }
}
#[cfg(test)]
mod test {
    use crate::char::CharExt;

    use crate::composer_model::delete_text::Direction;
    use crate::dom::nodes::text_node::CharType;
    use crate::tests::testutils_conversion::utf16;

    use super::{get_char_type, TextNode};

    #[test]
    fn get_char_type_for_whitespace() {
        // space
        assert_eq!(get_char_type('\u{0020}'), CharType::Whitespace);
        // no break space
        assert_eq!(get_char_type(char::nbsp()), CharType::Whitespace);
    }

    #[test]
    fn get_char_type_for_punctuation() {
        assert_eq!(get_char_type('='), CharType::Punctuation);
        assert_eq!(get_char_type('-'), CharType::Punctuation);
        assert_eq!(get_char_type('_'), CharType::Punctuation);
        assert_eq!(get_char_type('$'), CharType::Punctuation);
        assert_eq!(get_char_type('#'), CharType::Punctuation);
        assert_eq!(get_char_type('@'), CharType::Punctuation);
        assert_eq!(get_char_type('.'), CharType::Punctuation);
        assert_eq!(get_char_type(','), CharType::Punctuation);
    }

    #[test]
    fn get_char_type_for_other() {
        assert_eq!(get_char_type('1'), CharType::Other);
        assert_eq!(get_char_type('Q'), CharType::Other);
        assert_eq!(get_char_type('z'), CharType::Other);
    }

    #[test]
    fn offset_is_inside_node_end_of_node() {
        let test_node = TextNode::from(utf16("test"));
        assert!(!test_node.offset_is_inside_node(4, &Direction::Forwards));
        assert!(test_node.offset_is_inside_node(4, &Direction::Backwards))
    }

    #[test]
    fn offset_is_inside_node_start_of_node() {
        let test_node = TextNode::from(utf16("test"));
        assert!(test_node.offset_is_inside_node(0, &Direction::Forwards));
        assert!(!test_node.offset_is_inside_node(0, &Direction::Backwards));
    }

    #[test]
    fn offset_is_inside_node_middle_of_node() {
        let test_node = TextNode::from(utf16("test"));
        assert!(test_node.offset_is_inside_node(2, &Direction::Forwards));
        assert!(test_node.offset_is_inside_node(2, &Direction::Backwards));
    }

    #[test]
    fn pushing_text_node() {
        let mut t1 = TextNode::from(utf16("abc"));
        let t2 = TextNode::from(utf16("def"));
        t1.push(&t2);
        assert_eq!(t1, TextNode::from(utf16("abcdef")));
    }

    #[test]
    fn pushing_empty_text_node_does_nothing() {
        let mut t1 = TextNode::from(utf16("abc"));
        let t2 = TextNode::from(utf16(""));
        t1.push(&t2);
        assert_eq!(t1, TextNode::from(utf16("abc")));
    }

    #[test]
    fn slicing_text_node() {
        let mut text = TextNode::from(utf16("abcdefghi"));
        let after = text.slice_after(6);
        let mut before = text.slice_before(3);
        before.push(&text);
        before.push(&after);
        assert_eq!(before.data, "abcdefghi")
    }

    #[test]
    fn slicing_text_node_on_edge_does_nothing() {
        let mut text = TextNode::from(utf16("abcdefghi"));
        text.slice_after(9);
        text.slice_before(0);
        assert_eq!(text.data, "abcdefghi")
    }

    #[test]
    #[should_panic]
    fn slicing_after_edge_panics() {
        let mut text = TextNode::from(utf16("abcdefghi"));
        text.slice_after(42);
    }
}
