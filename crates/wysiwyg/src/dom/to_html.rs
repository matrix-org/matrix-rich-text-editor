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

use crate::dom::html_formatter::HtmlFormatter;

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
