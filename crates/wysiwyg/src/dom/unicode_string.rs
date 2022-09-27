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

    fn insert(&mut self, idx: usize, s: &Self::Str);
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

    // Should really be `-> Self::Chars<'a>`, but that requires GATs
    fn chars(&self) -> Box<dyn Iterator<Item = char> + '_>;
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
