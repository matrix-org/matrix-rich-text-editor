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

use std::marker::PhantomData;

use super::UnicodeString;

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
        let mut tree_part = S::from_str("");
        for i in 0..depth {
            if i == depth - 1 {
                if continuous_positions.contains(&i) {
                    tree_part
                        .push_string(&TreeSymbols::vertical_right_and_gt());
                } else {
                    tree_part.push_string(&TreeSymbols::up_right_and_gt());
                }
            } else {
                if continuous_positions.contains(&i) {
                    tree_part
                        .push_string(&TreeSymbols::vertical_and_whitespace());
                } else {
                    tree_part.push_string(&TreeSymbols::double_whitespace());
                }
            }
        }
        tree_part.push_string(&description);
        tree_part.push_string(&S::from_str("\n"));
        return tree_part;
    }
}

struct TreeSymbols<S>
where
    S: UnicodeString,
{
    phantom_data: PhantomData<S>,
}

impl<S> TreeSymbols<S>
where
    S: UnicodeString,
{
    fn double_whitespace() -> S {
        S::from_str("\u{0020}\u{0020}")
    }

    fn up_right_and_gt() -> S {
        S::from_str("\u{2514}\u{003E}")
    }

    fn vertical_right_and_gt() -> S {
        S::from_str("\u{251C}\u{003E}")
    }

    fn vertical_and_whitespace() -> S {
        S::from_str("\u{2502}\u{0020}")
    }
}
