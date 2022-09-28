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

use crate::Location;

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
