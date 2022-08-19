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

use crate::dom::nodes::ContainerNode;
use crate::dom::{Dom, HtmlFormatter};

use super::nodes::DomNode;

pub trait ToHtml<C>
where
    C: Clone,
{
    fn fmt_html(&self, f: &mut HtmlFormatter<C>);

    fn to_html(&self) -> Vec<C> {
        let mut f = HtmlFormatter::new();
        self.fmt_html(&mut f);
        f.finish()
    }
}

impl ToHtml<u16> for &str {
    fn fmt_html(&self, f: &mut HtmlFormatter<u16>) {
        f.write_iter(self.encode_utf16());
    }
}

impl ToHtml<u16> for String {
    fn fmt_html(&self, f: &mut HtmlFormatter<u16>) {
        f.write_iter(self.encode_utf16());
    }
}

impl<C> ToHtml<C> for Dom<C>
where
    C: Clone,
    ContainerNode<C>: ToHtml<C>,
{
    fn fmt_html(&self, f: &mut HtmlFormatter<C>) {
        self.document().fmt_html(f)
    }
}

fn fmt_node<C>(
    node: &ContainerNode<C>,
    lt: C,
    gt: C,
    equal: C,
    quote: C,
    space: C,
    fwd_slash: C,
    f: &mut HtmlFormatter<C>,
) where
    C: 'static + Clone,
    DomNode<C>: ToHtml<C>,
{
    let name = node.name();
    if !name.is_empty() {
        f.write_char(&lt);
        f.write(node.name());
        if let Some(attrs) = node.attributes() {
            for attr in attrs {
                f.write_char(&space);
                let (name, value) = attr;
                f.write(name);
                f.write_char(&equal);
                f.write_char(&quote);
                f.write(value);
                f.write_char(&quote);
            }
        }
        f.write_char(&gt);
    }

    for child in node.children() {
        child.fmt_html(f);
    }
    if !name.is_empty() {
        f.write_char(&lt);
        f.write_char(&fwd_slash);
        f.write(node.name());
        f.write_char(&gt);
    }
}

pub fn fmt_node_u16(node: &ContainerNode<u16>, f: &mut HtmlFormatter<u16>) {
    fmt_node(
        node, '<' as u16, '>' as u16, '=' as u16, '"' as u16, ' ' as u16,
        '/' as u16, f,
    );
}
