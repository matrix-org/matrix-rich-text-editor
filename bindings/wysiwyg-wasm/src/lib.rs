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

use std::collections::VecDeque;

use wasm_bindgen::prelude::*;
use widestring::Utf16String;

#[wasm_bindgen]
pub fn new_composer_model() -> ComposerModel {
    ComposerModel {
        inner: wysiwyg::ComposerModel::new(),
    }
}

#[wasm_bindgen]
pub fn new_composer_model_from_html(
    html: &str,
    start_utf16_codeunit: u32,
    end_utf16_codeunit: u32,
) -> ComposerModel {
    ComposerModel {
        inner: wysiwyg::ComposerModel::<Utf16String>::from_html(
            html,
            usize::try_from(start_utf16_codeunit).unwrap(),
            usize::try_from(end_utf16_codeunit).unwrap(),
        ),
    }
}

#[wasm_bindgen]
pub struct ComposerModel {
    inner: wysiwyg::ComposerModel<Utf16String>,
}

#[wasm_bindgen]
impl ComposerModel {
    pub fn new() -> Self {
        Self {
            inner: wysiwyg::ComposerModel::new(),
        }
    }

    pub fn from_example_format(text: &str) -> Self {
        Self {
            inner: wysiwyg::ComposerModel::from_example_format(text),
        }
    }

    pub fn to_example_format(&self) -> String {
        self.inner.to_example_format()
    }

    pub fn document(&self) -> DomHandle {
        DomHandle {
            inner: self.inner.state.dom.document().handle(),
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

    pub fn selection_start(&self) -> u32 {
        let ret: usize = self.inner.state.start.into();
        ret as u32
    }

    pub fn selection_end(&self) -> u32 {
        let ret: usize = self.inner.state.end.into();
        ret as u32
    }

    pub fn replace_text(&mut self, new_text: &str) -> ComposerUpdate {
        // Conversion here to UTF-16, which has presumably just been
        // converted to UTF-8 in the FFI bindings layer.
        // If the performance is a problem, we could fix this.
        // Internal task to track this: PSU-739
        ComposerUpdate::from(
            self.inner.replace_text(Utf16String::from_str(new_text)),
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

    pub fn italic(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.italic())
    }

    pub fn strike_through(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.strike_through())
    }

    pub fn underline(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.underline())
    }

    pub fn inline_code(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.inline_code())
    }

    pub fn undo(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.undo())
    }

    pub fn redo(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.redo())
    }

    pub fn ordered_list(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.ordered_list())
    }

    pub fn unordered_list(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.unordered_list())
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
    inner: wysiwyg::ComposerUpdate<Utf16String>,
}

impl ComposerUpdate {
    fn from(inner: wysiwyg::ComposerUpdate<Utf16String>) -> Self {
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
    pub select: Option<Selection>,
}

impl TextUpdate {
    pub fn from(inner: wysiwyg::TextUpdate<Utf16String>) -> Self {
        match inner {
            wysiwyg::TextUpdate::Keep => Self {
                keep: Some(Keep),
                replace_all: None,
                select: None,
            },
            wysiwyg::TextUpdate::ReplaceAll(r) => {
                let start_utf16_codeunit: usize = r.start.into();
                let end_utf16_codeunit: usize = r.end.into();
                Self {
                    keep: None,
                    replace_all: Some(ReplaceAll {
                        replacement_html: r.replacement_html.to_string(),
                        start_utf16_codeunit: u32::try_from(
                            start_utf16_codeunit,
                        )
                        .unwrap(),
                        end_utf16_codeunit: u32::try_from(end_utf16_codeunit)
                            .unwrap(),
                    }),
                    select: None,
                }
            }
            wysiwyg::TextUpdate::Select(s) => {
                let start_utf16_codeunit: usize = s.start.into();
                let end_utf16_codeunit: usize = s.end.into();
                Self {
                    keep: None,
                    replace_all: None,
                    select: Some(Selection {
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

#[derive(Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct Selection {
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
            // TODO: implement interface for MenuState
            wysiwyg::MenuState::Keep => Self {
                _none: Some(NoneMenuState),
            },
            _ => Self {
                _none: Some(NoneMenuState),
            },
        }
    }
}

#[wasm_bindgen]
pub struct NoneMenuState;

pub struct Dummy;

/// An iterator-like view of a DomHandle's children, written to work around
/// the lack of support for returning Vec<T> in wasm_bindgen.
#[wasm_bindgen]
pub struct DomChildren {
    inner: VecDeque<DomHandle>,
}

#[wasm_bindgen]
impl DomChildren {
    fn new() -> Self {
        Self {
            inner: VecDeque::new(),
        }
    }

    pub fn next(&mut self) -> Option<DomHandle> {
        self.inner.pop_front()
    }
}

impl FromIterator<DomHandle> for DomChildren {
    fn from_iter<T: IntoIterator<Item = DomHandle>>(iter: T) -> Self {
        Self {
            inner: VecDeque::from_iter(iter),
        }
    }
}

#[wasm_bindgen]
/// Refers to a node in the composer model.
pub struct DomHandle {
    inner: wysiwyg::DomHandle,
}

#[wasm_bindgen]
impl DomHandle {
    /// Returns "container", "line_break" or "text" depending on the type of
    /// node we refer to.
    /// Panics if we are not a valid reference (because the model has changed
    /// since we were created, or because you passed in a different model
    /// from the one that created us.)
    pub fn node_type(&self, model: &ComposerModel) -> String {
        let node = model.inner.state.dom.lookup_node(&self.inner);
        String::from(match node {
            wysiwyg::DomNode::Container(_) => "container",
            wysiwyg::DomNode::LineBreak(_) => "line_break",
            wysiwyg::DomNode::Text(_) => "text",
        })
    }

    /// Returns a list of our children nodes, or an empty list if we refer
    /// to a text or line break node.
    /// Panics if we are not a valid reference (because the model has changed
    /// since we were created, or because you passed in a different model
    /// from the one that created us.)
    pub fn children(&self, model: &ComposerModel) -> DomChildren {
        let node = model.inner.state.dom.lookup_node(&self.inner);
        match node {
            wysiwyg::DomNode::Container(node) => node
                .children()
                .iter()
                .map(|child| DomHandle {
                    inner: child.handle(),
                })
                .collect(),
            _ => DomChildren::new(),
        }
    }

    /// Returns the text of this node, or an empty string if this is a
    /// container or line break.
    /// Panics if we are not a valid reference (because the model has changed
    /// since we were created, or because you passed in a different model
    /// from the one that created us.)
    pub fn text(&self, model: &ComposerModel) -> String {
        let node = model.inner.state.dom.lookup_node(&self.inner);
        match node {
            wysiwyg::DomNode::Container(_) => String::from(""),
            wysiwyg::DomNode::LineBreak(_) => String::from(""),
            wysiwyg::DomNode::Text(node) => node.data().to_string(),
        }
    }

    /// Returns our tagname, or "-text-" if we are a text node.
    /// Panics if we are not a valid reference (because the model has changed
    /// since we were created, or because you passed in a different model
    /// from the one that created us.)
    pub fn tag(&self, model: &ComposerModel) -> String {
        let node = model.inner.state.dom.lookup_node(&self.inner);
        match node {
            wysiwyg::DomNode::Container(node) => node.name().to_string(),
            wysiwyg::DomNode::LineBreak(node) => node.name().to_string(),
            wysiwyg::DomNode::Text(_) => String::from("-text-"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::ComposerModel;

    #[test]
    fn can_find_types_of_nodes() {
        let mut model = ComposerModel::new();
        model.replace_text("foo");

        assert_eq!(model.document().node_type(&model), "container");
        assert_eq!(
            model.document().children(&model).inner[0].node_type(&model),
            "text"
        );
    }

    #[test]
    fn can_enumerate_children_of_nodes() {
        let mut model = ComposerModel::new();
        model.replace_text("01234");
        model.select(1, 4);
        model.bold();
        model.select(2, 3);
        model.italic();

        let children = model.document().children(&model).inner;
        let grandchildren = children[1].children(&model).inner;
        let great_grandchildren = grandchildren[1].children(&model).inner;

        assert_eq!(children[0].node_type(&model), "text");
        assert_eq!(children[0].text(&model), "0");
        assert_eq!(children[1].node_type(&model), "container");
        assert_eq!(children[1].tag(&model), "strong");
        assert_eq!(grandchildren[0].node_type(&model), "text");
        assert_eq!(grandchildren[0].text(&model), "1");
        assert_eq!(grandchildren[1].node_type(&model), "container");
        assert_eq!(grandchildren[1].tag(&model), "em");
        assert_eq!(great_grandchildren[0].node_type(&model), "text");
        assert_eq!(great_grandchildren[0].text(&model), "2");
        assert_eq!(grandchildren[2].node_type(&model), "text");
        assert_eq!(grandchildren[2].text(&model), "3");
        assert_eq!(children[2].node_type(&model), "text");
        assert_eq!(children[2].text(&model), "4");
    }
}
