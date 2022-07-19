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

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn new_composer_model() -> ComposerModel {
    ComposerModel {
        inner: wysiwyg::ComposerModel::new(),
    }
}

#[wasm_bindgen]
pub struct ComposerModel {
    inner: wysiwyg::ComposerModel<u16>,
}

#[wasm_bindgen]
impl ComposerModel {
    pub fn new() -> Self {
        Self {
            inner: wysiwyg::ComposerModel::new(),
        }
    }

    pub fn select(
        &mut self,
        start_utf16_codeunit: u32,
        end_utf16_codeunit: u32,
    ) {
        self.inner.select(
            wysiwyg::Location::from(
                usize::try_from(start_utf16_codeunit).unwrap(),
            ),
            wysiwyg::Location::from(
                usize::try_from(end_utf16_codeunit).unwrap(),
            ),
        );
    }

    pub fn replace_text(&mut self, new_text: &str) -> ComposerUpdate {
        // TODO: conversion here to UTF-16, which has presumably just been
        // converted to UTF-8 in the bindings layer!
        // If the performance is a problem, we could fix this.
        ComposerUpdate::from(
            self.inner
                .replace_text(&new_text.encode_utf16().collect::<Vec<_>>()),
        )
    }

    pub fn enter(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.enter())
    }

    pub fn backspace(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.backspace())
    }

    pub fn delete(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.delete())
    }

    pub fn bold(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.bold())
    }

    /*pub fn action_response(
        self: &Arc<Self>,
        action_id: String,
        response: ActionResponse,
    ) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(
            self.inner
                .lock()
                .unwrap()
                .action_response(action_id, response.into()),
        ))
    }*/
}

#[wasm_bindgen]
pub struct ComposerUpdate {
    inner: wysiwyg::ComposerUpdate<u16>,
}

impl ComposerUpdate {
    fn from(inner: wysiwyg::ComposerUpdate<u16>) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen]
impl ComposerUpdate {
    pub fn text_update(&self) -> TextUpdate {
        TextUpdate::from(self.inner.text_update.clone())
    }

    pub fn menu_state(&self) -> MenuState {
        MenuState::from(self.inner.menu_state.clone())
    }

    /*pub fn actions(&self) -> Vec<ComposerAction> {
        self.inner
            .actions
            .iter()
            .map(|action| ComposerAction::from(action.clone()))
            .collect()
    }*/
}

#[wasm_bindgen(getter_with_clone)]
pub struct TextUpdate {
    pub keep: Option<Keep>,
    pub replace_all: Option<ReplaceAll>,
}

impl TextUpdate {
    pub fn from(inner: wysiwyg::TextUpdate<u16>) -> Self {
        match inner {
            wysiwyg::TextUpdate::Keep => Self {
                keep: Some(Keep),
                replace_all: None,
            },
            wysiwyg::TextUpdate::ReplaceAll(r) => {
                let start_utf16_codeunit: usize = r.start.into();
                let end_utf16_codeunit: usize = r.end.into();
                Self {
                    keep: None,
                    replace_all: Some(ReplaceAll {
                        replacement_html: String::from_utf16(
                            &r.replacement_html,
                        )
                        .expect("Model returned invalid UTF-16"),
                        start_utf16_codeunit: u32::try_from(
                            start_utf16_codeunit,
                        )
                        .unwrap(),
                        end_utf16_codeunit: u32::try_from(end_utf16_codeunit)
                            .unwrap(),
                    }),
                }
            }
        }
    }
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct Keep;

#[derive(Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct ReplaceAll {
    pub replacement_html: String,
    pub start_utf16_codeunit: u32,
    pub end_utf16_codeunit: u32,
}

#[wasm_bindgen]
pub struct MenuState {
    _none: Option<NoneMenuState>,
}

impl MenuState {
    pub fn from(inner: wysiwyg::MenuState) -> Self {
        match inner {
            wysiwyg::MenuState::None => Self {
                _none: Some(NoneMenuState),
            },
        }
    }
}

#[wasm_bindgen]
pub struct NoneMenuState;

#[wasm_bindgen]
pub struct ComposerAction {
    inner: wysiwyg::ComposerAction,
}

#[wasm_bindgen]
impl ComposerAction {
    pub fn action_id(&self) -> String {
        self.inner.action_id.clone()
    }

    pub fn action(&self) -> ActionRequest {
        ActionRequest::from(self.inner.action.clone())
    }
}

#[wasm_bindgen]
pub struct ActionRequest {
    _dummy: Option<Dummy>,
}

impl ActionRequest {
    pub fn from(inner: wysiwyg::ActionRequest) -> Self {
        match inner {
            wysiwyg::ActionRequest::Dummy => Self {
                _dummy: Some(Dummy),
            },
        }
    }
}

#[wasm_bindgen]
pub struct ActionResponse {
    _dummy: Option<Dummy>,
}

impl ActionResponse {
    /*
    fn into(self) -> wysiwyg::ActionResponse {
        if let Some(_dummy) = self._dummy {
            wysiwyg::ActionResponse::Dummy
        } else {
            panic!("Unknown ActionResponse type");
        }
    }*/
}

pub struct Dummy;
