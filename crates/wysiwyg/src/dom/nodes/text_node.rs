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
use crate::dom::to_html::ToHtml;
use crate::dom::to_markdown::{MarkdownError, MarkdownOptions, ToMarkdown};
use crate::dom::to_raw_text::ToRawText;
use crate::dom::to_tree::ToTree;
use crate::dom::unicode_string::{UnicodeStr, UnicodeStrExt, UnicodeStringExt};
use crate::dom::UnicodeString;
use html_escape;

const ZWSP: &str = "\u{200B}";
const ZWSP_char: char = '\u{200B}';

// categories of character for backspace/delete word
#[derive(PartialEq, Debug)]
pub enum CharType {
    Whitespace,
    Linebreak,
    Punctuation,
    Other,
}
#[derive(Clone, Debug, PartialEq)]
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

    pub fn set_data(&mut self, data: S) {
        self.data = data;
    }

    pub fn handle(&self) -> DomHandle {
        self.handle.clone()
    }

    pub fn set_handle(&mut self, handle: DomHandle) {
        self.handle = handle;
    }

    pub fn is_blank(&self) -> bool {
        self.data
            .chars()
            .all(|c| matches!(c, ' ' | '\x09'..='\x0d'))
    }

    /// Add a leading ZWSP to the text node if it doesn't already
    /// has one. Return true if the operation is executed.
    pub fn add_leading_zwsp(&mut self) -> bool {
        let text = self.data.to_string();
        if !text.starts_with(ZWSP) {
            let mut new_text: S = ZWSP.into();
            new_text.push(self.data());
            self.set_data(new_text);
            true
        } else {
            false
        }
    }

    /// Remove a leading ZWSP from the text node if it has one.
    /// Return true if the operation is executed.
    pub fn remove_leading_zwsp(&mut self) -> bool {
        let mut text = self.data().to_string();
        if text.starts_with(ZWSP) {
            text.remove(0);
            self.set_data(text.into());
            true
        } else {
            false
        }
    }

    /// This gets the character at the cursor, considering the
    /// direction of travel
    pub fn char_at_offset(
        &self,
        offset: usize,
        direction: &Direction,
    ) -> Option<char> {
        self.data()
            .chars()
            .nth(direction.get_index_from_cursor(offset))
    }

    /// This gets the character type at the cursor, considering the
    /// direction of travel
    pub fn char_type_at_offset(
        &self,
        offset: usize,
        direction: &Direction,
    ) -> Option<CharType> {
        let char = self.char_at_offset(offset, direction);
        match char {
            Some(c) => Some(get_char_type(c)),
            None => None,
        }
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
    pub fn has_length(&self) -> bool {
        self.data().len() != 0
    }
}

/// Given a character, determine it's type
fn get_char_type(c: char) -> CharType {
    // in order to determine where a ctrl/opt + delete type operation finishes
    // we need to distinguish between whitespace (nb no newline characters), punctuation
    // and then everything else is treated as the same type
    if c.is_whitespace() || c == ZWSP_char {
        // manually add zero width space character
        return CharType::Whitespace;
    }

    if c.is_ascii_punctuation() {
        return CharType::Punctuation;
    }

    return CharType::Other;
}

impl<S> ToHtml<S> for TextNode<S>
where
    S: UnicodeString,
{
    fn fmt_html(
        &self,
        buf: &mut S,
        selection_writer: Option<&mut SelectionWriter>,
        is_last_node_in_parent: bool,
    ) {
        let cur_pos = buf.len();
        let string = self.data.to_string();

        let mut escaped = html_escape::encode_text(&string)
            // Replace all pairs of spaces with non-breaking ones. Transforms
            // `a     b` to `a\u{A0}\u{A0}\u{A0}\u{A0} b`, which will render
            // exactly as five spaces like in the input.
            .replace("  ", "\u{A0}\u{A0}");
        if is_last_node_in_parent
            && escaped.chars().next_back().map_or(false, |c| c == ' ')
        {
            // If this is the last node and it ends in a space, replace that
            // space with a non-breaking one.
            escaped.replace_range(escaped.len() - 1.., "\u{A0}");
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

impl<S> ToTree<S> for TextNode<S>
where
    S: UnicodeString,
{
    fn to_tree_display(&self, continuous_positions: Vec<usize>) -> S {
        let mut description = S::from("\"");
        let text = &self.data.to_string().replace('\u{200b}', "~");
        description.push(text.as_str());
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
        _options: &MarkdownOptions,
    ) -> Result<(), MarkdownError<S>> {
        buffer.push(self.data.to_owned());

        Ok(())
    }
}

mod test {
    use crate::dom::nodes::text_node::{CharType, ZWSP_char};

    use super::get_char_type;

    #[test]
    fn get_char_type_for_whitespace() {
        // space
        assert_eq!(get_char_type('\u{0020}'), CharType::Whitespace);
        // no break space
        assert_eq!(get_char_type('\u{00A0}'), CharType::Whitespace);
        // zero width space
        assert_eq!(get_char_type(ZWSP_char), CharType::Whitespace);
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
}
