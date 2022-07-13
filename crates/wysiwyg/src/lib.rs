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

pub struct ComposerModel {}

impl ComposerModel {
    pub fn new() -> Self {
        Self {}
    }

    /**
     * Cursor is at end_codepoint.
     */
    pub fn select(&mut self, start_codepoint: usize, end_codepoint: usize) {
        drop(start_codepoint);
        drop(end_codepoint);
    }

    pub fn replace_text(&mut self, new_text: String) -> ComposerUpdate {
        drop(new_text);
        ComposerUpdate::keep(MenuState::None)
    }

    pub fn backspace(&mut self) -> ComposerUpdate {
        ComposerUpdate::keep(MenuState::None)
    }

    pub fn delete(&mut self) -> ComposerUpdate {
        ComposerUpdate::keep(MenuState::None)
    }

    pub fn action_response(
        &mut self,
        action_id: String,
        response: ActionResponse,
    ) -> ComposerUpdate {
        drop(action_id);
        drop(response);
        ComposerUpdate::keep(MenuState::None)
    }
}

#[derive(Debug, Clone)]
pub struct ComposerUpdate {
    pub text_update: TextUpdate,
    pub menu_state: MenuState,
    pub actions: Vec<ComposerAction>,
}

impl ComposerUpdate {
    pub fn keep(menu_state: MenuState) -> Self {
        Self {
            text_update: TextUpdate::Keep,
            menu_state,
            actions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TextUpdate {
    Keep,
    ReplaceAll(String),
}

#[derive(Debug, Clone)]
pub enum MenuState {
    None,
}

#[derive(Debug, Clone)]
pub struct ComposerAction {
    pub action_id: String,
    pub action: ActionRequest,
}

#[derive(Debug, Clone)]
pub enum ActionRequest {
    Dummy,
}

#[derive(Debug, Clone)]
pub enum ActionResponse {
    Dummy,
}
