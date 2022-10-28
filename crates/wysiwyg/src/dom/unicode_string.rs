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
use std::fmt;
use std::iter;
use std::ops::{Deref, Index, Range, RangeFrom, RangeTo};

use widestring::{Utf16Str, Utf16String, Utf32Str, Utf32String};

/// The type of string being used inside a [Dom] instance. Must
/// contain valid Unicode, and allow slicing by code unit positions.
/// We implement this for String, Utf16String and Utf32String (from the
/// widestring crate).
pub trait UnicodeString:
    Clone
    + fmt::Debug
    + fmt::Display
    + Default
    + PartialEq
    + AsRef<[Self::CodeUnit]>
    + for<'a> From<&'a str>
    + From<String>
    + Deref<Target = Self::Str>
    + for<'a> Extend<&'a Self::Str>
    + Extend<Self>
    + Extend<char>
    + for<'a> Extend<&'a str>
    + for<'a> Extend<&'a Self::Str>
    + Index<Range<usize>, Output = Self::Str>
    + Index<RangeFrom<usize>, Output = Self::Str>
    + Index<RangeTo<usize>, Output = Self::Str>
{
    type CodeUnit: Copy + From<u8> + PartialEq;
    type Str: UnicodeStr<CodeUnit = Self::CodeUnit, Owned = Self> + ?Sized;

    fn insert(&mut self, idx: usize, s: &Self::Str);
}

pub trait UnicodeStr:
    fmt::Display
    + fmt::Debug
    + PartialEq
    + PartialEq<str>
    + AsRef<[Self::CodeUnit]>
    + ToOwned
    + Index<Range<usize>, Output = Self>
    + Index<RangeFrom<usize>, Output = Self>
    + Index<RangeTo<usize>, Output = Self>
{
    type CodeUnit: Copy + From<u8> + PartialEq;

    // Should really be `-> Self::Chars<'a>`, but that requires GATs
    fn chars(&self) -> Box<dyn Iterator<Item = char> + '_>;

    /// Returns the length of the char in indices of the current encoding
    fn char_len(&self, char: &char) -> usize;

    /// Splits the current UnicodeStr into graphemes (visual characters), from start to end indexes
    fn graphemes(&self, start: usize, end: usize) -> Vec<&Self> {
        let mut ret = Vec::new();
        let mut start = start;
        let mut offset = 0;
        let mut chars = self.chars().peekable();
        while chars.peek().is_some() {
            let c = chars.next().unwrap();
            let char_len = self.char_len(&c);
            let new_offset = offset + char_len;
            if offset >= start && c != '\u{200D}' {
                // Omit ZWJ character, just update the current offset
                if let Some(next_char) = chars.peek() {
                    if *next_char != '\u{200D}' {
                        // If next char is no ZWJ, push the current grapheme and update start
                        ret.push(&self[start..new_offset]);
                        start = new_offset;
                    }
                    // If next char is ZWJ, we just update the offset so the grapheme keeps growing
                } else {
                    // Last char, push the current grapheme
                    ret.push(&self[start..new_offset]);
                }
            }
            if new_offset >= end {
                break;
            } else {
                offset = new_offset;
            }
        }
        ret
    }
}

impl UnicodeString for String {
    type CodeUnit = u8;
    type Str = str;

    fn insert(&mut self, idx: usize, s: &Self::Str) {
        self.insert_str(idx, s);
    }
}

impl UnicodeStr for str {
    type CodeUnit = u8;

    fn chars(&self) -> Box<dyn Iterator<Item = char> + '_> {
        Box::new(self.chars())
    }

    fn char_len(&self, char: &char) -> usize {
        char.len_utf8()
    }
}

impl UnicodeString for Utf16String {
    type CodeUnit = u16;
    type Str = Utf16Str;

    fn insert(&mut self, idx: usize, s: &Self::Str) {
        self.insert_utfstr(idx, s);
    }
}

impl UnicodeStr for Utf16Str {
    type CodeUnit = u16;

    fn chars(&self) -> Box<dyn Iterator<Item = char> + '_> {
        Box::new(self.chars())
    }

    fn char_len(&self, char: &char) -> usize {
        char.len_utf16()
    }
}

impl UnicodeString for Utf32String {
    type CodeUnit = u32;
    type Str = Utf32Str;

    fn insert(&mut self, idx: usize, s: &Self::Str) {
        self.insert_utfstr(idx, s);
    }
}

impl UnicodeStr for Utf32Str {
    type CodeUnit = u32;

    fn chars(&self) -> Box<dyn Iterator<Item = char> + '_> {
        Box::new(self.chars())
    }

    fn char_len(&self, _: &char) -> usize {
        // 1 char == 1 u32, see https://doc.rust-lang.org/std/primitive.char.html#method.from_u32
        1
    }
}

pub trait UnicodeStringExt: UnicodeString {
    fn push<T>(&mut self, s: T)
    where
        Self: Extend<T>;
}

impl<S: UnicodeString> UnicodeStringExt for S {
    fn push<T>(&mut self, s: T)
    where
        Self: Extend<T>,
    {
        self.extend(iter::once(s))
    }
}

pub trait UnicodeStrExt: UnicodeStr {
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
}

impl<S: UnicodeStr + ?Sized> UnicodeStrExt for S {
    fn is_empty(&self) -> bool {
        self.as_ref().is_empty()
    }

    fn len(&self) -> usize {
        self.as_ref().len()
    }
}

#[cfg(test)]
mod test {
    use crate::dom::unicode_string::UnicodeStr;
    use widestring::{Utf16String, Utf32String};

    #[test]
    fn test_emoji_utf8() {
        let str = "😄";
        let graphemes = str.graphemes(0, str.len());
        assert_eq!(1, graphemes.len());
    }

    #[test]
    fn test_emoji_complex_utf8() {
        let str = "😮‍💨";
        let graphemes = str.graphemes(0, str.len());
        assert_eq!(1, graphemes.len());
    }

    #[test]
    #[should_panic]
    fn test_index_inside_char_with_emoji_utf8() {
        let str = "😮‍💨";
        str.graphemes(1, str.len());
    }

    #[test]
    fn test_indexes_out_of_range_with_emoji_utf8() {
        let str = "😮‍💨";
        let graphemes = str.graphemes(10, str.len());
        assert!(graphemes.is_empty());
    }

    #[test]
    fn test_emoji_complex_with_text_utf8() {
        let str = "Test 😮‍💨";
        let graphemes = str.graphemes(0, str.len());
        // [ 'T', 'e', 's', 't', ' ', '😮‍💨' ]
        assert_eq!(6, graphemes.len());
    }

    #[test]
    fn test_emoji_complex_with_text_utf16() {
        let str = Utf16String::from_str("Test 😮‍💨");
        let graphemes = str.graphemes(0, str.len());
        assert_eq!(6, graphemes.len());
    }

    #[test]
    #[should_panic]
    fn test_index_inside_char_with_emoji_utf16() {
        let str = Utf16String::from_str("😮‍💨");
        str.graphemes(1, str.len());
    }

    #[test]
    fn test_indexes_out_of_range_with_emoji_utf16() {
        let str = Utf16String::from_str("😮‍💨");
        let graphemes = str.graphemes(10, str.len());
        assert!(graphemes.is_empty());
    }

    #[test]
    fn test_emoji_complex_with_text_utf32() {
        let str = Utf32String::from_str("Test 😮‍💨");
        let graphemes = str.graphemes(0, str.len());
        // [ 'T', 'e', 's', 't', ' ', '😮‍💨' ]
        assert_eq!(6, graphemes.len());
    }

    #[test]
    fn test_indexes_out_of_range_with_emoji_utf32() {
        let str = Utf32String::from_str("😮‍💨");
        let graphemes = str.graphemes(10, str.len());
        assert!(graphemes.is_empty());
    }
}
