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

use unicode_string::{UnicodeString, UnicodeStringExt};

const DOUBLE_WHITESPACE: &str = "\u{0020}\u{0020}";
const UP_RIGHT_AND_GT: &str = "\u{2514}\u{003E}";
const VERTICAL_RIGHT_AND_GT: &str = "\u{251C}\u{003E}";
const VERTICAL_AND_WHITESPACE: &str = "\u{2502}\u{0020}";

pub trait ToTree<S>
where
    S: UnicodeString,
{
    /// Output tree representation from current item.
    fn to_tree(&self) -> S {
        self.to_tree_display(vec![])
    }

    /// Output tree representation from current item with
    /// given vector of ancestors that have extra children.
    fn to_tree_display(&self, continuous_positions: Vec<usize>) -> S;

    /// Output content of a tree line with given description
    /// with current depth in the tree as well as positions
    /// of ancestor that have extra children.
    fn tree_line(
        &self,
        description: S,
        depth: usize,
        continuous_positions: Vec<usize>,
    ) -> S {
        let mut tree_part = S::default();
        for i in 0..depth {
            if i == depth - 1 {
                if continuous_positions.contains(&i) {
                    tree_part.push(VERTICAL_RIGHT_AND_GT);
                } else {
                    tree_part.push(UP_RIGHT_AND_GT);
                }
            } else if continuous_positions.contains(&i) {
                tree_part.push(VERTICAL_AND_WHITESPACE);
            } else {
                tree_part.push(DOUBLE_WHITESPACE);
            }
        }
        tree_part.push(description);
        tree_part.push('\n');
        tree_part
    }
}
