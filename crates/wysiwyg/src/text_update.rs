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

use crate::{dom::UnicodeString, Location};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextUpdate<S>
where
    S: UnicodeString,
{
    Keep,
    PanicRecovery(PanicRecovery<S>),
    ReplaceAll(ReplaceAll<S>),
    Select(Selection),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanicRecovery<S>
where
    S: UnicodeString,
{
    pub previous_html: S,
    pub start: Location,
    pub end: Location,
    pub error_message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceAll<S>
where
    S: UnicodeString,
{
    pub replacement_html: S,
    pub start: Location,
    pub end: Location,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub start: Location,
    pub end: Location,
}
