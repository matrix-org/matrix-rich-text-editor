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
pub struct Utf16CodeunitLocation(usize);

impl Utf16CodeunitLocation {
    pub fn codepoint(&self, s: &str) -> CodepointLocation {
        let first_part_as_utf16: Vec<u16> =
            s.encode_utf16().take(self.0).collect();
        let first_part = String::from_utf16(&first_part_as_utf16).unwrap();

        CodepointLocation::end_of(&first_part)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

impl From<usize> for Utf16CodeunitLocation {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod test {
    use super::Utf16CodeunitLocation;

    #[test]
    fn ascii_converts_to_equal_codepoint_location() {
        let s = "abcdefgh";
        assert_eq!(Utf16CodeunitLocation::from(0).codepoint(&s).as_usize(), 0);
        assert_eq!(Utf16CodeunitLocation::from(1).codepoint(&s).as_usize(), 1);
        assert_eq!(Utf16CodeunitLocation::from(2).codepoint(&s).as_usize(), 2);
        assert_eq!(Utf16CodeunitLocation::from(8).codepoint(&s).as_usize(), 8);
        assert_eq!(Utf16CodeunitLocation::from(9).codepoint(&s).as_usize(), 8);
        assert_eq!(Utf16CodeunitLocation::from(20).codepoint(&s).as_usize(), 8);
    }
}
