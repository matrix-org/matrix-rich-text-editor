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

use std::ops;

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Default)]
pub struct Location(usize);

impl From<usize> for Location {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<Location> for usize {
    fn from(val: Location) -> Self {
        val.0
    }
}

impl PartialEq<usize> for Location {
    fn eq(&self, other: &usize) -> bool {
        self.0 == *other
    }
}

impl ops::Add for Location {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl ops::AddAssign<isize> for Location {
    fn add_assign(&mut self, rhs: isize) {
        let mut i = isize::try_from(self.0).expect("Location was too large!");
        i += rhs;
        if i < 0 {
            i = 0;
        }
        self.0 = usize::try_from(i).unwrap();
    }
}

impl ops::SubAssign<isize> for Location {
    fn sub_assign(&mut self, rhs: isize) {
        *self += -rhs
    }
}
