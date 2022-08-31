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

mod composer_action;
mod composer_list_handler;
mod composer_model;
mod composer_state;
mod composer_update;
mod dom;
mod format_nodes;
mod format_type;
mod location;
mod menu_state;
mod node_joiner;
mod tests;
mod text_update;

pub use crate::composer_action::ActionRequest;
pub use crate::composer_action::ActionResponse;
pub use crate::composer_action::ComposerAction;
pub use crate::composer_model::ComposerModel;
pub use crate::composer_state::ComposerState;
pub use crate::composer_update::ComposerUpdate;
pub use crate::dom::ToHtml;
pub use crate::dom::ToRawText;
pub use crate::dom::UnicodeString;
pub use crate::format_type::InlineFormatType;
pub use crate::location::Location;
pub use crate::menu_state::MenuState;
use crate::node_joiner::NodeJoiner;
pub use crate::text_update::ReplaceAll;
pub use crate::text_update::TextUpdate;
