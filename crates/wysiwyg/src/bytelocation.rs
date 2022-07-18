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

use crate::CodepointLocation;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ByteLocation(usize);

impl ByteLocation {
    pub fn codepoint(&self, s: &str) -> CodepointLocation {
        let mut i = 0;
        let mut cp = 0;
        while i < self.0 && i < s.len() {
            cp += 1;
            i += 1;
            while !s.is_char_boundary(i) {
                i += 1;
            }
        }
        CodepointLocation::from(cp)
    }

    pub(crate) fn as_usize(&self) -> usize {
        self.0
    }
}

impl From<usize> for ByteLocation {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod test {
    use super::ByteLocation;

    #[test]
    fn codepoint_of_point_off_end_is_end() {
        let loc = ByteLocation::from(20);
        assert_eq!(loc.codepoint("foo").as_usize(), 3);
    }
}
