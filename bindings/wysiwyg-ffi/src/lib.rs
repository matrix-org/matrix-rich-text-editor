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

uniffi_macros::include_scaffolding!("wysiwyg_composer");

mod ffi_action_state;
mod ffi_composer_action;
mod ffi_composer_model;
mod ffi_composer_state;
mod ffi_composer_update;
mod ffi_menu_state;
mod ffi_text_update;
mod into_ffi;

use std::sync::Arc;

pub use crate::ffi_action_state::ActionState;
pub use crate::ffi_composer_action::ComposerAction;
pub use crate::ffi_composer_model::ComposerModel;
pub use crate::ffi_composer_state::ComposerState;
pub use crate::ffi_composer_update::ComposerUpdate;
pub use crate::ffi_menu_state::MenuState;
pub use crate::ffi_text_update::TextUpdate;

pub fn new_composer_model() -> Arc<ComposerModel> {
    Arc::new(ComposerModel::new())
}
