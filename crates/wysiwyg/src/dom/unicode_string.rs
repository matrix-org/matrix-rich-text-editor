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

use crate::char::CharExt;
use std::fmt;
use std::iter;
use std::ops::{Deref, Index, Range, RangeFrom, RangeTo};
use unicode_segmentation::UnicodeSegmentation;

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

    /// Creates a new unicode string consisting of a single ZWSP.
    fn zwsp() -> Self {
        char::zwsp().to_string().into()
    }
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
    type StringType: UnicodeString;

    // Should really be `-> Self::Chars<'a>`, but that requires GATs
    fn chars(&self) -> Box<dyn Iterator<Item = char> + '_>; // can call this to go through the chars

    /// Returns the length of the char in indices of the current encoding
    fn char_len(&self, char: &char) -> usize;
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
    type StringType = String;

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
    type StringType = Utf16String;

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
    type StringType = Utf32String;

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
    fn find_graphemes_at(
        &self,
        index: usize,
    ) -> (Option<Self::StringType>, Option<Self::StringType>);
    fn u8_map_index(&self, pos: usize) -> usize;
}

impl<S: UnicodeStr + ?Sized> UnicodeStrExt for S {
    fn is_empty(&self) -> bool {
        self.as_ref().is_empty()
    }

    fn len(&self) -> usize {
        self.as_ref().len()
    }

    /// Assuming [index] is a boundary between graphemes, returns a pair with the previous and next
    /// graphemes, if present.
    fn find_graphemes_at(
        &self,
        index: usize,
    ) -> (Option<Self::StringType>, Option<Self::StringType>) {
        let u8_str = self.to_string();
        let u8_index = self.u8_map_index(index);
        let graphemes = u8_str.grapheme_indices(true);
        let mut prev = None;
        let mut next = None;
        for g in graphemes {
            if g.0 == u8_index {
                next = Some(Self::StringType::from(g.1))
            }
            let length = g.1.len();
            if g.0 + length == u8_index {
                prev = Some(Self::StringType::from(g.1))
            }
        }
        (prev, next)
    }

    /// Translates indexes from any [UnicodeString] implementation to UTF-8.
    fn u8_map_index(&self, pos: usize) -> usize {
        let mut offset_u8: usize = 0;
        let mut offset_orig: usize = 0;
        let mut pos_u8 = usize::MAX;
        for char in self.chars() {
            let cur_offset = offset_orig;
            offset_orig += self.char_len(&char);
            if pos_u8 == usize::MAX && cur_offset >= pos {
                pos_u8 = offset_u8;
                break;
            }
            offset_u8 += char.len_utf8();
        }
        if pos_u8 == usize::MAX {
            if offset_orig >= pos {
                pos_u8 = offset_u8;
            } else {
                panic!("UTF-8 index is out of bounds.");
            }
        }
        pos_u8
    }
}

#[cfg(test)]
mod test {
    use crate::dom::unicode_string::UnicodeStrExt;
    use widestring::{Utf16String, Utf32String};

    #[test]
    fn test_emoji_utf8() {
        let str = "😄";
        let (prev, next) = str.find_graphemes_at(0);
        assert!(prev.is_none());
        assert_eq!("😄", next.unwrap());
    }

    #[test]
    fn test_emoji_complex_utf8() {
        let str = "😮‍💨";
        let (prev, next) = str.find_graphemes_at(0);
        assert!(prev.is_none());
        assert_eq!("😮‍💨", next.unwrap());
    }

    #[test]
    fn test_index_inside_char_with_emoji_utf8() {
        let str = "😮‍💨";
        let (prev, next) = str.find_graphemes_at(1);
        assert!(prev.is_none());
        assert!(next.is_none());
    }

    #[test]
    #[should_panic]
    fn test_indexes_out_of_range_with_emoji_utf8() {
        let str = "😮‍💨";
        str.find_graphemes_at(100);
    }

    #[test]
    fn test_emoji_complex_with_text_utf8() {
        let str = "Test 😮‍💨";
        let (prev, next) = str.find_graphemes_at(5);
        assert_eq!(" ", prev.unwrap());
        assert_eq!("😮‍💨", next.unwrap());
    }

    #[test]
    fn test_emoji_complex_with_text_utf16() {
        let str = Utf16String::from_str("Test 😮‍💨");
        let (prev, next) = str.find_graphemes_at(5);
        assert_eq!(" ", prev.unwrap());
        assert_eq!("😮‍💨", next.unwrap());
    }

    #[test]
    fn test_index_inside_char_with_emoji_utf16() {
        let str = Utf16String::from_str("😮‍💨");
        let (prev, next) = str.find_graphemes_at(1);
        assert!(prev.is_none());
        assert!(next.is_none());
    }

    #[test]
    #[should_panic]
    fn test_indexes_out_of_range_with_emoji_utf16() {
        let str = Utf16String::from_str("😮‍💨");
        str.find_graphemes_at(100);
    }

    #[test]
    fn test_emoji_complex_with_text_utf32() {
        let str = Utf32String::from_str("Test 😮‍💨");
        let (prev, next) = str.find_graphemes_at(5);
        assert_eq!(" ", prev.unwrap());
        assert_eq!("😮‍💨", next.unwrap());
    }

    #[test]
    #[should_panic]
    fn test_indexes_out_of_range_with_emoji_utf32() {
        let str = Utf32String::from_str("😮‍💨");
        str.find_graphemes_at(100);
    }
}
