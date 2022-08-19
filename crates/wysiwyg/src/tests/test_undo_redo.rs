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

#![cfg(test)]

use crate::tests::testutils_composer_model::cm;

use crate::dom::nodes::{DomNode, TextNode};
use crate::{InlineFormatType, ToHtml};

#[test]
fn undoing_action_restores_previous_state() {
    let mut model = cm("hello |");
    let mut prev = model.state.clone();
    let prev_text_node =
        TextNode::from("world!".encode_utf16().collect::<Vec<u16>>());
    prev.dom.append(DomNode::Text(prev_text_node));
    model.previous_states.push(prev.clone());

    model.undo();

    assert_eq!(prev.dom.children().len(), model.state.dom.children().len());
}

#[test]
fn inserting_text_creates_previous_state() {
    let mut model = cm("|");
    assert!(model.previous_states.is_empty());

    model.replace_text(&"hello world!".to_html());
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

    model.format(InlineFormatType::Bold);
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
