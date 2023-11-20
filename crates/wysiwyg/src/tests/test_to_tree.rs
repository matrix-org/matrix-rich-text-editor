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

use crate::tests::testutils_composer_model::cm;
use crate::ToTree;

#[test]
fn single_nested_tag_produces_tree() {
    let model = cm("<b>abc<i>def</i></b>|");
    assert_eq!(
        model.state.dom.to_tree(),
        r#"
└>b
  ├>"abc"
  └>i
    └>"def"
"#,
    );
}

#[test]
fn multiple_tags_nested_inside_one_produce_tree() {
    let model =
        cm("<ul><li>ab</li><li><b>cd</b></li><li><i><b>ef|</b></i></li></ul>");
    assert_eq!(
        model.state.dom.to_tree(),
        r#"
└>ul
  ├>li
  │ └>"ab"
  ├>li
  │ └>b
  │   └>"cd"
  └>li
    └>i
      └>b
        └>"ef"
"#,
    );
}

#[test]
fn br_within_text_shows_up_in_tree() {
    let model = cm("a<br />|b");
    assert_eq!(
        model.state.dom.to_tree(),
        r#"
├>p
│ └>"a"
└>p
  └>"b"
"#,
    );
}

#[test]
fn link_href_shows_up_in_tree() {
    let model = cm("Some <a href=\"https://matrix.org\">url|</a>");
    assert_eq!(
        model.state.dom.to_tree(),
        r#"
├>"Some "
└>a "https://matrix.org"
  └>"url"
"#,
    );
}

#[test]
fn mention_shows_up_in_tree() {
    let model =
        cm("Some <a href=\"https://matrix.to/#/@test:example.org\">test</a>|");
    assert_eq!(
        model.state.dom.to_tree(),
        r#"
├>"Some "
└>mention "test", https://matrix.to/#/@test:example.org
"#,
    );
}
