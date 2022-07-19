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

use crate::{ByteLocation, CodepointDelta};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CodepointLocation(usize);

impl CodepointLocation {
    pub fn byte(&self, s: &str) -> ByteLocation {
        let mut i = 0;
        let mut cp = 0;
        while i < s.len() {
            if cp == self.0 {
                return ByteLocation::from(i);
            }
            cp += 1;
            i += 1;
            while !s.is_char_boundary(i) {
                i += 1;
            }
        }
        ByteLocation::from(s.len())
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }

    /**
     * Add `delta` to the current codepoint value.
     * If the sum is negative or too big to be a usize, the current value
     * is set to 0.
     */
    pub fn move_forward(&mut self, delta: CodepointDelta) {
        let mut value = isize::try_from(self.0).unwrap();
        value += delta.as_isize();
        self.0 = usize::try_from(value).unwrap_or(0);
    }

    pub fn as_delta(&self) -> CodepointDelta {
        CodepointDelta::from(isize::try_from(self.as_usize()).unwrap())
    }

    pub(crate) fn end_of(s: &str) -> CodepointLocation {
        ByteLocation::from(s.len()).codepoint(s)
    }
}

impl From<usize> for CodepointLocation {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod test {
    use super::CodepointLocation;

    #[test]
    fn byte_of_point_off_end_is_end() {
        let loc = CodepointLocation::from(20);
        assert_eq!(loc.byte("foo").as_usize(), 3);
    }
}
