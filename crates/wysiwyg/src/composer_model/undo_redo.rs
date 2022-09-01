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

use crate::{ComposerModel, ComposerUpdate, UnicodeString};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn undo(&mut self) -> ComposerUpdate<S> {
        if let Some(prev) = self.previous_states.pop() {
            self.next_states.push(self.state.clone());
            self.state = prev;
            self.create_update_replace_all()
        } else {
            ComposerUpdate::keep()
        }
    }

    pub fn redo(&mut self) -> ComposerUpdate<S> {
        if let Some(next) = self.next_states.pop() {
            self.previous_states.push(self.state.clone());
            self.state = next;
            self.create_update_replace_all()
        } else {
            ComposerUpdate::keep()
        }
    }

    pub(crate) fn push_state_to_history(&mut self) {
        // Clear future events as they're no longer valid
        self.next_states.clear();
        // Store a copy of the current state in the previous_states
        self.previous_states.push(self.state.clone());
    }
}
