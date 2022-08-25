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

use widestring::{Utf16String, Utf32String};

/// The type of string being used inside a [Dom] instance. Must
/// contain valid Unicode, and allow slicing by code unit positions.
/// We implement this for String, Utf16String and Utf32String (from the
/// widestring crate).
pub trait UnicodeString: Clone {
    type CodeUnit: Clone;

    fn new() -> Self;

    fn from_str<T: AsRef<str> + ?Sized>(s: &T) -> Self;

    fn from_vec(v: impl Into<Vec<Self::CodeUnit>>) -> Result<Self, String>;

    fn is_empty(&self) -> bool;

    /// Convert this character to a code unit.
    /// Panics if this character requires more than one code unit
    fn c_from_char(ch: char) -> Self::CodeUnit;

    fn as_slice(&self) -> &[Self::CodeUnit];

    fn to_utf8(&self) -> String;

    fn push_string(&mut self, s: &Self);

    fn len(&self) -> usize;
}

impl UnicodeString for String {
    type CodeUnit = u8;

    fn new() -> Self {
        String::new()
    }

    fn from_str<T: AsRef<str> + ?Sized>(s: &T) -> Self {
        String::from(s.as_ref())
    }

    fn from_vec(v: impl Into<Vec<Self::CodeUnit>>) -> Result<Self, String> {
        String::from_utf8(v.into()).map_err(|e| e.to_string())
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn c_from_char(ch: char) -> Self::CodeUnit {
        assert!(ch.len_utf8() == 1);
        let mut buf = [0; 1];
        ch.encode_utf8(&mut buf);
        buf[0]
    }

    fn as_slice(&self) -> &[Self::CodeUnit] {
        self.as_bytes()
    }

    fn to_utf8(&self) -> String {
        self.clone()
    }

    fn push_string(&mut self, s: &Self) {
        self.push_str(&s)
    }

    fn len(&self) -> usize {
        self.len()
    }
}

impl UnicodeString for Utf16String {
    type CodeUnit = u16;

    fn new() -> Self {
        Utf16String::new()
    }

    fn from_str<T: AsRef<str> + ?Sized>(s: &T) -> Self {
        Utf16String::from_str(s)
    }

    fn from_vec(v: impl Into<Vec<Self::CodeUnit>>) -> Result<Self, String> {
        Utf16String::from_vec(v.into()).map_err(|e| e.to_string())
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn c_from_char(ch: char) -> Self::CodeUnit {
        let mut ret = Utf16String::new();
        ret.push(ch);
        assert!(ret.len() == 1);
        ret.into_vec()[0]
    }

    fn as_slice(&self) -> &[Self::CodeUnit] {
        self.as_slice()
    }

    fn to_utf8(&self) -> String {
        // Unwrap can't fail since we encode as UTF-8.
        String::from_utf8(self.encode_utf8().collect()).unwrap()
    }

    fn push_string(&mut self, s: &Self) {
        self.push_utfstr(s.as_utfstr())
    }

    fn len(&self) -> usize {
        self.len()
    }
}

impl UnicodeString for Utf32String {
    type CodeUnit = u32;

    fn new() -> Self {
        Utf32String::new()
    }

    fn from_str<T: AsRef<str> + ?Sized>(s: &T) -> Self {
        Utf32String::from_str(s)
    }

    fn from_vec(v: impl Into<Vec<Self::CodeUnit>>) -> Result<Self, String> {
        Utf32String::from_vec(v.into()).map_err(|e| e.to_string())
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn c_from_char(ch: char) -> Self::CodeUnit {
        let mut ret = Utf32String::new();
        ret.push(ch);
        assert!(ret.len() == 1);
        ret.into_vec()[0]
    }

    fn as_slice(&self) -> &[Self::CodeUnit] {
        self.as_slice()
    }

    fn to_utf8(&self) -> String {
        // Unwrap can't fail since we encode as UTF-8.
        String::from_utf8(self.encode_utf8().collect()).unwrap()
    }

    fn push_string(&mut self, s: &Self) {
        self.push_utfstr(s.as_utfstr())
    }

    fn len(&self) -> usize {
        self.len()
    }
}
