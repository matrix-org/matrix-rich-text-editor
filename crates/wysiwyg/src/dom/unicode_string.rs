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

use std::iter;
use std::ops::{Deref, Index, Range, RangeFrom, RangeTo};

use widestring::{Utf16Str, Utf16String, Utf32Str, Utf32String};

/// The type of string being used inside a [Dom] instance. Must
/// contain valid Unicode, and allow slicing by code unit positions.
/// We implement this for String, Utf16String and Utf32String (from the
/// widestring crate).
pub trait UnicodeString:
    Clone
    + std::fmt::Debug
    + std::fmt::Display
    + Default
    + PartialEq
    + AsRef<[Self::CodeUnit]>
    + for<'a> From<&'a str>
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

    fn from_vec(v: impl Into<Vec<Self::CodeUnit>>) -> Result<Self, String>;

    /// Convert this character to a code unit.
    /// Panics if this character requires more than one code unit
    fn c_from_char(ch: char) -> Self::CodeUnit;
}

pub trait UnicodeStr:
    std::fmt::Display
    + PartialEq
    + PartialEq<str>
    + AsRef<[Self::CodeUnit]>
    + ToOwned
    + Index<Range<usize>, Output = Self>
    + Index<RangeFrom<usize>, Output = Self>
    + Index<RangeTo<usize>, Output = Self>
{
    type CodeUnit: Copy + From<u8> + PartialEq;
}

impl UnicodeString for String {
    type CodeUnit = u8;
    type Str = str;

    fn from_vec(v: impl Into<Vec<Self::CodeUnit>>) -> Result<Self, String> {
        String::from_utf8(v.into()).map_err(|e| e.to_string())
    }

    fn c_from_char(ch: char) -> Self::CodeUnit {
        assert!(ch.len_utf8() == 1);
        let mut buf = [0; 1];
        ch.encode_utf8(&mut buf);
        buf[0]
    }
}

impl UnicodeStr for str {
    type CodeUnit = u8;
}

impl UnicodeString for Utf16String {
    type CodeUnit = u16;
    type Str = Utf16Str;

    fn from_vec(v: impl Into<Vec<Self::CodeUnit>>) -> Result<Self, String> {
        Utf16String::from_vec(v.into()).map_err(|e| e.to_string())
    }

    fn c_from_char(ch: char) -> Self::CodeUnit {
        let mut ret = Utf16String::new();
        ret.push(ch);
        assert!(ret.len() == 1);
        ret.into_vec()[0]
    }
}

impl UnicodeStr for Utf16Str {
    type CodeUnit = u16;
}

impl UnicodeString for Utf32String {
    type CodeUnit = u32;
    type Str = Utf32Str;

    fn from_vec(v: impl Into<Vec<Self::CodeUnit>>) -> Result<Self, String> {
        Utf32String::from_vec(v.into()).map_err(|e| e.to_string())
    }

    fn c_from_char(ch: char) -> Self::CodeUnit {
        let mut ret = Utf32String::new();
        ret.push(ch);
        assert!(ret.len() == 1);
        ret.into_vec()[0]
    }
}

impl UnicodeStr for Utf32Str {
    type CodeUnit = u32;
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
