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

use crate::dom::UnicodeString;
use crate::{ComposerAction, Location, MenuState, ReplaceAll, TextUpdate};

#[derive(Debug, Clone)]
pub struct ComposerUpdate<S>
where
    S: UnicodeString,
{
    pub text_update: TextUpdate<S>,
    pub menu_state: MenuState,
    pub actions: Vec<ComposerAction>,
}

impl<S> ComposerUpdate<S>
where
    S: UnicodeString,
{
    pub fn keep() -> Self {
        Self {
            text_update: TextUpdate::<S>::Keep,
            menu_state: MenuState::Keep,
            actions: Vec::new(),
        }
    }

    pub fn update_menu(menu_state: MenuState) -> Self {
        Self {
            text_update: TextUpdate::<S>::Keep,
            menu_state: menu_state,
            actions: Vec::new(),
        }
    }

    pub fn replace_all(
        replacement_html: S,
        start: Location,
        end: Location,
        menu_state: MenuState,
    ) -> Self {
        Self {
            text_update: TextUpdate::ReplaceAll(ReplaceAll {
                replacement_html,
                start,
                end,
            }),
            menu_state: menu_state,
            actions: Vec::new(),
        }
    }
}
