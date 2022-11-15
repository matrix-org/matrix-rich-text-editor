use std::collections::HashMap;

use crate::{ActionState, ComposerAction};

pub trait IntoFfi {
    fn into_ffi(self) -> HashMap<ComposerAction, ActionState>;
}

impl IntoFfi for &HashMap<wysiwyg::ComposerAction, wysiwyg::ActionState> {
    fn into_ffi(self) -> HashMap<ComposerAction, ActionState> {
        self.iter().map(|(a, s)| (a.into(), s.into())).collect()
    }
}
