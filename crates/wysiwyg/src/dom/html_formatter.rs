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

use super::UnicodeString;

pub struct HtmlFormatter<S>
where
    S: UnicodeString,
{
    chars: Vec<S::CodeUnit>,
    known_char_data: KnownCharData<S>,
}

pub enum HtmlChar {
    Equal,
    ForwardSlash,
    Gt,
    Lt,
    Quote,
    Space,
}

impl<S> HtmlFormatter<S>
where
    S: UnicodeString,
{
    pub fn new() -> Self {
        Self {
            chars: Vec::new(),
            known_char_data: KnownCharData::new(),
        }
    }

    pub fn write_char(&mut self, c: HtmlChar) {
        self.chars.push(match c {
            HtmlChar::Equal => self.known_char_data.equal.clone(),
            HtmlChar::ForwardSlash => {
                self.known_char_data.forward_slash.clone()
            }
            HtmlChar::Gt => self.known_char_data.gt.clone(),
            HtmlChar::Lt => self.known_char_data.lt.clone(),
            HtmlChar::Quote => self.known_char_data.quote.clone(),
            HtmlChar::Space => self.known_char_data.space.clone(),
        });
    }

    pub fn write(&mut self, slice: &[S::CodeUnit]) {
        self.chars.extend_from_slice(slice);
    }

    pub fn write_iter(&mut self, chars: impl Iterator<Item = S::CodeUnit>) {
        self.chars.extend(chars)
    }

    pub fn write_vec(&mut self, chars: Vec<S::CodeUnit>) {
        self.chars.extend(chars)
    }

    pub fn finish(self) -> S {
        S::from_vec(self.chars).expect("Unable to convert to unicode!")
    }
}

struct KnownCharData<S>
where
    S: UnicodeString,
{
    equal: S::CodeUnit,
    forward_slash: S::CodeUnit,
    gt: S::CodeUnit,
    lt: S::CodeUnit,
    quote: S::CodeUnit,
    space: S::CodeUnit,
}

impl<S> KnownCharData<S>
where
    S: UnicodeString,
{
    fn new() -> Self {
        Self {
            equal: S::c_from_char('='),
            forward_slash: S::c_from_char('/'),
            gt: S::c_from_char('>'),
            lt: S::c_from_char('<'),
            quote: S::c_from_char('"'),
            space: S::c_from_char(' '),
        }
    }
}
