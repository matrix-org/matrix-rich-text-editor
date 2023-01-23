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

use crate::tests::testutils_composer_model::{cm, tx};

use crate::dom::nodes::{DomNode, TextNode};
use crate::{InlineFormatType, Location};

use crate::tests::testutils_conversion::utf16;

#[test]
fn undoing_action_restores_previous_state() {
    let mut model = cm("hello |");
    let mut prev = model.state.clone();
    let prev_text_node = TextNode::from(utf16("world!"));
    prev.dom
        .append_at_end_of_document(DomNode::Text(prev_text_node));
    model.previous_states.push(prev.clone());

    model.undo();

    assert_eq!(prev.dom.children().len(), model.state.dom.children().len());
}

#[test]
fn inserting_text_creates_previous_state() {
    let mut model = cm("|");
    assert!(model.previous_states.is_empty());

    model.replace_text(utf16("hello world!"));
    assert!(!model.previous_states.is_empty());
}

#[test]
fn backspacing_text_creates_previous_state() {
    let mut model = cm("hello world!|");
    assert!(model.previous_states.is_empty());

    model.backspace();
    assert!(!model.previous_states.is_empty());
}

#[test]
fn deleting_text_creates_previous_state() {
    let mut model = cm("hello |world!");
    assert!(model.previous_states.is_empty());

    model.delete();
    assert!(!model.previous_states.is_empty());
}

#[test]
fn formatting_text_creates_previous_state() {
    let mut model = cm("hello {world}|!");
    assert!(model.previous_states.is_empty());

    model.bold();
    assert!(!model.previous_states.is_empty());
}

#[test]
fn undoing_action_removes_last_previous_state() {
    let mut model = cm("hello {world}|!");
    model.previous_states.push(model.state.clone());

    model.undo();

    assert!(model.previous_states.is_empty());
}

#[test]
fn undoing_action_adds_popped_state_to_next_states() {
    let mut model = cm("hello {world}|!");
    model.previous_states.push(model.state.clone());

    model.undo();

    assert_eq!(model.next_states[0], model.state);
}

#[test]
fn redo_pops_state_from_next_states() {
    let mut model = cm("hello {world}|!");
    model.next_states.push(model.state.clone());

    model.redo();

    assert!(model.next_states.is_empty());
}

#[test]
fn redoing_action_adds_popped_state_to_previous_states() {
    let mut model = cm("hello {world}|!");
    model.next_states.push(model.state.clone());

    model.redo();

    assert_eq!(model.previous_states[0], model.state);
}

#[test]
fn undoing_restores_toggled_format_types() {
    let mut model = cm("|");
    model.bold();
    model.italic();
    assert_eq!(
        model.state.toggled_format_types,
        Vec::from([InlineFormatType::Bold, InlineFormatType::Italic,])
    );
    model.replace_text(utf16("a"));
    assert_eq!(model.state.toggled_format_types, Vec::new());
    model.undo();
    assert_eq!(
        model.state.toggled_format_types,
        Vec::from([InlineFormatType::Bold, InlineFormatType::Italic,])
    );
}

#[test]
#[allow(deprecated)]
fn can_undo_adding_line_break() {
    let mut model = cm("Test|");
    model.add_line_break();
    model.undo();
    assert_eq!(tx(&model), "Test|");
}

#[test]
fn can_undo_pressing_enter() {
    let mut model = cm("Test|");
    model.enter();
    model.undo();
    assert_eq!(tx(&model), "Test|");
}

#[test]
fn can_undo_with_selection() {
    let mut model = cm("Test{foo}|bar");
    model.enter();
    model.undo();
    assert_eq!(tx(&model), "Test{foo}|bar");
}

#[test]
fn deleting_a_selection_with_enter_only_adds_one_to_undo_stack() {
    let mut model = cm("Test{foo}|bar");

    // Do 2 things
    model.replace_text(utf16("baz"));
    model.select(Location::from(4), Location::from(7));
    model.enter();

    // Undo twice
    model.undo();
    model.undo();

    // We should be back where we started
    assert_eq!(tx(&model), "Test{foo}|bar");
}

#[test]
fn undoing_enter_only_undoes_one() {
    let mut model = cm("Test|");
    model.enter();
    assert_eq!(tx(&model), "<p>Test</p><p>|</p>");
    model.enter();
    assert_eq!(tx(&model), "<p>Test</p><p>&nbsp;</p><p>|</p>");
    model.undo();
    assert_eq!(tx(&model), "<p>Test</p><p>|</p>");
}

#[test]
fn replacing_text_with_newlines_only_adds_one_to_undo_stack() {
    let mut model = cm("abc|");
    model.replace_text(utf16("def\nghi"));
    model.replace_text(utf16("\njkl\n"));
    model.undo();
    model.undo();
    assert_eq!(tx(&model), "abc|");
}
