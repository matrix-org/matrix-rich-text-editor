use std::collections::{HashMap, HashSet};

use strum::IntoEnumIterator;
use wysiwyg::UnicodeString;

use crate::{ActionState, ComposerAction};

pub enum MenuState {
    Keep,
    Update {
        action_states: HashMap<ComposerAction, ActionState>,
    },
}

impl MenuState {
    pub fn from(inner: wysiwyg::MenuState) -> Self {
        match inner {
            wysiwyg::MenuState::Keep => Self::Keep,
            wysiwyg::MenuState::Update(menu_update) => Self::Update {
                action_states: build_action_states(
                    &menu_update.reversed_actions,
                    &menu_update.disabled_actions,
                ),
            },
        }
    }
}

pub fn build_composer_action_states<S: UnicodeString>(
    inner: &wysiwyg::ComposerModel<S>,
) -> HashMap<ComposerAction, ActionState> {
    build_action_states(&inner.reversed_actions, &inner.disabled_actions)
}

fn build_action_states(
    reversed_actions: &HashSet<wysiwyg::ComposerAction>,
    disabled_actions: &HashSet<wysiwyg::ComposerAction>,
) -> HashMap<ComposerAction, ActionState> {
    ComposerAction::iter()
        .map(|action_type| {
            (
                action_type,
                action_value(&action_type, reversed_actions, disabled_actions),
            )
        })
        .collect()
}

fn action_value(
    action_type: &ComposerAction,
    reversed_actions: &HashSet<wysiwyg::ComposerAction>,
    disabled_actions: &HashSet<wysiwyg::ComposerAction>,
) -> ActionState {
    let action_type = action_type.into();
    if reversed_actions.contains(&action_type) {
        ActionState::Reversed
    } else if disabled_actions.contains(&action_type) {
        ActionState::Disabled
    } else {
        ActionState::Enabled
    }
}
