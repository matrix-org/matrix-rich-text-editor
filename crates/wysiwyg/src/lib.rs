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

pub struct ComposerModel {
    html: String, // TODO: not an AST yet!
    selection_start_codepoint: usize,
    selection_end_codepoint: usize,
}

impl ComposerModel {
    pub fn new() -> Self {
        Self {
            html: String::from(""),
            selection_start_codepoint: 0,
            selection_end_codepoint: 0,
        }
    }

    pub fn create_update_replace_all(&self) -> ComposerUpdate {
        ComposerUpdate::replace_all(
            self.html.clone(),
            self.selection_start_codepoint,
            self.selection_end_codepoint,
        )
    }

    /**
     * TODO: just a hack
     */
    fn do_bold(&mut self) {
        let mut range =
            [self.selection_start_codepoint, self.selection_end_codepoint];
        range.sort();

        self.html = format!(
            "{}<strong>{}</strong>{}",
            &self.html[..range[0]],
            &self.html[range[0]..range[1]],
            &self.html[range[1]..]
        );
    }

    /**
     * Cursor is at end_codepoint.
     */
    pub fn select(&mut self, start_codepoint: usize, end_codepoint: usize) {
        self.selection_start_codepoint = start_codepoint;
        self.selection_end_codepoint = end_codepoint;
    }

    pub fn replace_text(&mut self, new_text: String) -> ComposerUpdate {
        self.html += &new_text; // TODO: just a hack
        self.selection_start_codepoint += 1;
        self.selection_end_codepoint += 1;

        // TODO: for now, we replace every time, to check ourselves, but
        // at least some of the time we should not
        self.create_update_replace_all()
        //ComposerUpdate::keep()
    }

    pub fn enter(&mut self) -> ComposerUpdate {
        ComposerUpdate::keep()
    }

    pub fn backspace(&mut self) -> ComposerUpdate {
        ComposerUpdate::keep()
    }

    pub fn delete(&mut self) -> ComposerUpdate {
        ComposerUpdate::keep()
    }

    pub fn bold(&mut self) -> ComposerUpdate {
        self.do_bold();
        self.create_update_replace_all()
    }

    pub fn action_response(
        &mut self,
        action_id: String,
        response: ActionResponse,
    ) -> ComposerUpdate {
        drop(action_id);
        drop(response);
        ComposerUpdate::keep()
    }
}

#[derive(Debug, Clone)]
pub struct ComposerUpdate {
    pub text_update: TextUpdate,
    pub menu_state: MenuState,
    pub actions: Vec<ComposerAction>,
}

impl ComposerUpdate {
    pub fn keep() -> Self {
        Self {
            text_update: TextUpdate::Keep,
            menu_state: MenuState::None,
            actions: Vec::new(),
        }
    }

    pub fn replace_all(
        replacement_html: String,
        selection_start_codepoint: usize,
        selection_end_codepoint: usize,
    ) -> Self {
        Self {
            text_update: TextUpdate::ReplaceAll(ReplaceAll {
                replacement_html,
                selection_start_codepoint,
                selection_end_codepoint,
            }),
            menu_state: MenuState::None,
            actions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TextUpdate {
    Keep,
    ReplaceAll(ReplaceAll),
}

#[derive(Debug, Clone)]
pub struct ReplaceAll {
    pub replacement_html: String,
    pub selection_start_codepoint: usize,
    pub selection_end_codepoint: usize,
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
