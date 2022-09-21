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

use std::ops::Range;

use crate::composer_model::example_format::SelectionWriter;
use crate::{DomNode, ToHtml};

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

    pub fn write_at(&mut self, pos: usize, slice: &[S::CodeUnit]) {
        self.chars.splice(pos..pos, slice.to_vec());
    }

    pub fn write_iter(&mut self, chars: impl Iterator<Item = S::CodeUnit>) {
        self.chars.extend(chars)
    }

    pub fn write_vec(&mut self, chars: Vec<S::CodeUnit>) {
        self.chars.extend(chars)
    }

    pub fn write_node(
        &mut self,
        node: &DomNode<S>,
        is_last: bool,
        selection_writer: Option<&mut SelectionWriter>,
    ) {
        let cur_pos = self.chars.len();
        let mut ret: Vec<S::CodeUnit> = Vec::new();
        match node {
            DomNode::Text(t) => {
                let mut post_processed_data = String::new();
                html_escape::encode_text_to_string(
                    t.data().to_utf8(),
                    &mut post_processed_data,
                );
                handle_several_whitespaces(&mut post_processed_data, is_last);
                self.write(S::from_str(&post_processed_data).as_slice());
                if let Some(sel_writer) = selection_writer {
                    sel_writer.write_node(self, cur_pos, node);
                }
            }
            DomNode::Container(c) => {
                c.fmt_html(self, selection_writer);
            }
            DomNode::LineBreak(n) => {
                ret.push(S::c_from_char('<'));
                ret.extend_from_slice(n.name().as_slice());
                ret.push(S::c_from_char(' '));
                ret.push(S::c_from_char('/'));
                ret.push(S::c_from_char('>'));
                self.write_vec(ret);
                if let Some(sel_writer) = selection_writer {
                    sel_writer.write_node(self, cur_pos, node);
                }
            }
        }
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

fn handle_several_whitespaces(html: &mut String, is_last: bool) {
    let mut ranges_to_replace = Vec::new();
    let mut current_range: Range<usize> = usize::MAX..usize::MAX;
    let mut whitespaces: usize = 0;
    let mut needs_to_replace = false;
    for (i, c) in html.chars().enumerate() {
        if c == ' ' || c == '\u{A0}' {
            if c == ' ' {
                needs_to_replace = true;
            }
            if current_range.start == usize::MAX {
                whitespaces = 1;
                current_range.start = i;
            } else {
                whitespaces += 1;
            }
        } else {
            if needs_to_replace && whitespaces > 1 {
                current_range.end = i;
                ranges_to_replace.push((current_range.clone(), whitespaces));
            }
            needs_to_replace = false;
            current_range = usize::MAX..usize::MAX;
        }
    }
    if is_last && needs_to_replace {
        current_range.end = html.len();
        ranges_to_replace.push((current_range.clone(), whitespaces));
    }

    for (range, whitespaces) in ranges_to_replace.iter().rev() {
        let replacement: String = (0..*whitespaces).map(|_| '\u{A0}').collect();
        html.replace_range(range.clone(), &replacement);
    }
}
