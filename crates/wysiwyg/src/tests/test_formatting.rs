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
use crate::tests::testutils_conversion::utf16;
use widestring::Utf16String;

use crate::InlineFormatType::Bold;
use crate::Location;
use crate::{ComposerModel, InlineFormatType};

#[test]
fn selecting_and_bolding_multiple_times() {
    let mut model = cm("aabbcc|");
    model.select(Location::from(0), Location::from(2));
    model.bold();
    model.select(Location::from(4), Location::from(6));
    model.bold();
    assert_eq!(
        &model.state.dom.to_string(),
        "<strong>aa</strong>bb<strong>cc</strong>"
    );
}

#[test]
fn bolding_ascii_adds_strong_tags() {
    let mut model = cm("aa{bb}|cc");
    model.bold();
    assert_eq!(tx(&model), "aa<strong>{bb}|</strong>cc");

    let mut model = cm("aa|{bb}cc");
    model.bold();
    assert_eq!(tx(&model), "aa<strong>|{bb}</strong>cc");
}

#[test]
fn format_several_nodes_with_empty_text_nodes() {
    let mut model = cm("{some}| different nodes");
    model.bold();
    model.select(Location::from(5), Location::from(14));
    model.italic();
    model.select(Location::from(2), Location::from(17));
    model.strike_through();
    assert_eq!(tx(&model), "<strong>so<del>{me</del></strong><del>&nbsp;</del><em><del>different</del></em><del> no}|</del>des")
}

#[test]
fn selecting_and_unbolding_multiple_times() {
    let mut model = cm("<strong>aabbcc|</strong>");
    model.select(Location::from(0), Location::from(2));
    model.bold();
    model.select(Location::from(4), Location::from(6));
    model.bold();
    assert_eq!(tx(&model), "aa<strong>bb</strong>{cc}|");
}

#[test]
fn unformat_nested_node() {
    let mut model = cm("aa<em>b<strong>{bc}|</strong></em>c");
    model.bold();
    assert_eq!(tx(&model), "aa<em>b{bc}|</em>c");
}

#[test]
fn partial_unformat_nested_node() {
    let mut model = cm("aa<em>b<strong>b{c}|</strong></em>c");
    model.bold();
    assert_eq!(tx(&model), "aa<em>b<strong>b</strong>{c}|</em>c");
}

#[test]
fn unformat_toplevel_node_moves_nested_nodes() {
    let mut model = cm("aa<em>{b<strong>bc}|</strong></em>c");
    model.italic();
    assert_eq!(tx(&model), "aa{b<strong>bc}|</strong>c");
}

#[test]
fn partial_unformat_toplevel_node_reconstructs_expected_model() {
    let mut model = cm("aa<em>b<strong>b{c}|</strong></em>c");
    model.italic();
    assert_eq!(tx(&model), "aa<em>b</em><strong><em>b</em>{c}|</strong>c");
}

#[test]
fn unformat_several_nodes() {
    let mut model = cm("<strong>so<del>me</del></strong><del> </del><em><del>different</del></em><del> no</del>des|");
    model.select(Location::from(2), Location::from(17));
    model.strike_through();
    model.select(Location::from(5), Location::from(14));
    model.italic();
    model.select(Location::from(0), Location::from(4));
    model.bold();
    assert_eq!(tx(&model), "{some}| different nodes");
}

#[test]
fn formatting_twice_adds_no_formatting() {
    let input = "a{aabbbcc}|c";
    let mut model = cm(input);
    for _i in 0..=1 {
        model.bold();
        model.italic();
        model.strike_through();
        model.underline();
    }
    assert_eq!(tx(&model), input);
}

#[test]
fn formatting_nested_format_nodes_and_line_breaks() {
    let mut model =
        cm("aa<strong>a</strong><strong><br />{bbb<br />}|cc</strong>c");
    model.italic();
    assert_eq!(
        tx(&model),
        "aa<strong>a<br /><em>{bbb<br />}|</em>cc</strong>c"
    );
}

#[test]
fn formatting_deeper_nested_format_nodes_and_nested_line_breaks() {
    let mut model =
        cm("aa<strong>a</strong><strong><u><br />{b</u>bb<br />}|cc</strong>c");
    model.italic();
    assert_eq!(
        tx(&model),
        "aa<strong>a<u><br /><em>{b</em></u><em>bb<br />}|</em>cc</strong>c"
    );
}

#[test]
fn formatting_with_zero_length_selection_apply_on_replace_text() {
    let mut model = cm("aaa|bbb");
    model.bold();
    model.italic();
    model.underline();
    assert_eq!(tx(&model), "aaa|bbb");
    assert_eq!(
        model.state.toggled_format_types,
        Vec::from([
            InlineFormatType::Bold,
            InlineFormatType::Italic,
            InlineFormatType::Underline
        ])
    );
    model.replace_text(utf16("ccc"));
    assert_eq!(tx(&model), "aaa<strong><em><u>ccc|</u></em></strong>bbb");
}

#[test]
fn unformatting_with_zero_length_selection_removes_on_replace_text() {
    let mut model = cm("<strong>aaa|bbb</strong>");
    model.bold();
    assert_eq!(
        model.state.toggled_format_types,
        Vec::from([InlineFormatType::Bold]),
    );
    model.replace_text(utf16("ccc"));
    assert_eq!(tx(&model), "<strong>aaa</strong>ccc|<strong>bbb</strong>");
}

#[test]
fn formatting_and_unformatting_with_zero_length_selection() {
    let mut model = cm("<em>aaa|bbb</em>");
    model.bold();
    model.italic();
    model.replace_text(utf16("ccc"));
    assert_eq!(tx(&model), "<em>aaa</em><strong>ccc|</strong><em>bbb</em>");
}

#[test]
fn selecting_removes_toggled_format_types() {
    let mut model = cm("aaa|");
    model.bold();
    assert_eq!(
        model.state.toggled_format_types,
        Vec::from([InlineFormatType::Bold]),
    );
    model.select(Location::from(2), Location::from(2));
    assert_eq!(model.state.toggled_format_types, Vec::new(),);
    model.replace_text(utf16("ccc"));
    assert_eq!(tx(&model), "aaccc|a");
}

#[test]
fn formatting_again_removes_toggled_format_type() {
    let mut model = cm("aaa|");
    model.bold();
    assert_eq!(
        model.state.toggled_format_types,
        Vec::from([InlineFormatType::Bold]),
    );
    model.bold();
    assert_eq!(model.state.toggled_format_types, Vec::new(),);
}

#[test]
fn unformatting_consecutive_same_formatting_nodes() {
    let mut model = cm("{<strong>Test</strong><strong> </strong><strong>test</strong><strong> test</strong>}|");
    model.bold();
    assert_eq!(tx(&model), "{Test test test}|");
}

#[test]
fn unformatting_consecutive_same_formatting_nodes_with_nested_line_break() {
    let mut model = cm("{<strong>Test</strong><strong> </strong><strong>te<br />st</strong><strong> test</strong>}|");
    model.bold();
    assert_eq!(tx(&model), "{Test te<br />st test}|");
}

#[test]
fn unformatting_consecutive_same_formatting_nodes_with_nested_node() {
    let mut model = cm("{<strong>Test</strong><strong> </strong><strong>t<em>es</em>t</strong><strong> test</strong>}|");
    model.bold();
    assert_eq!(tx(&model), "{Test t<em>es</em>t test}|");
}

#[test]
fn format_empty_model_applies_formatting() {
    let mut model = ComposerModel::<Utf16String>::new();
    model.bold();
    assert!(model.state.toggled_format_types.contains(&Bold));
}

#[test]
fn changing_selection_to_same_doesnt_removes_formatting_state() {
    let mut model = cm("AAA | BBB");
    model.bold();
    model.select(Location::from(4), Location::from(4));
    assert!(model.state.toggled_format_types.contains(&Bold));
}

#[test]
fn formatting_before_typing_anything_applies_formatting() {
    let mut model = cm("|");
    model.bold();
    model.replace_text(utf16("d"));
    assert_eq!(tx(&model), "<strong>d|</strong>");
}

#[test]
fn formatting_in_an_empty_model_applies_formatting() {
    let mut model = ComposerModel::new();
    model.bold();
    model.replace_text(utf16("d"));
    assert_eq!(tx(&model), "<strong>d|</strong>");
}
