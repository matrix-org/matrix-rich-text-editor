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

pub struct HtmlFormatter<C> {
    chars: Vec<C>,
}

impl<C> HtmlFormatter<C>
where
    C: Clone,
{
    pub fn new() -> Self {
        Self { chars: Vec::new() }
    }

    pub fn write_char(&mut self, c: &C) {
        self.chars.push(c.clone());
    }

    pub fn write(&mut self, slice: &[C]) {
        self.chars.extend_from_slice(slice);
    }

    pub fn write_iter(&mut self, chars: impl Iterator<Item = C>) {
        self.chars.extend(chars)
    }

    pub fn finish(self) -> Vec<C> {
        self.chars
    }
}
