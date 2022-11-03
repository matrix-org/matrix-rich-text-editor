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

use pulldown_cmark as md_parser;

use crate::UnicodeString;

pub struct MarkdownHTMLParser {}

impl MarkdownHTMLParser {
    pub fn to_html<S>(markdown: &S) -> S
    where
        S: UnicodeString,
    {
        use md_parser::{html::push_html as compile_to_html, Options, Parser};

        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);

        let markdown = markdown.to_string();

        let parser = Parser::new_ext(&markdown, options);

        let mut html = String::new();
        compile_to_html(&mut html, parser);

        // By default, there is a `<p>…</p>\n` around the HTML content. That's the
        // correct way to handle a text block in Markdown. But it breaks our
        // assumption regarding the HTML markup. So let's remove it.
        let html = {
            if !html.starts_with("<p>") {
                &html[..]
            } else {
                let p = "<p>".len();
                let ppnl = "</p>\n".len();

                &html[p..html.len() - ppnl]
            }
        };

        // Format new lines.
        let html = html
            .replace("<ul>\n", "<ul>")
            .replace("</ul>\n", "</ul>")
            .replace("<ol>\n", "<ol>")
            .replace("</ol>\n", "</ol>")
            .replace("</li>\n", "</li>")
            .replace("<br />\n", "<br />");

        S::try_from(html).unwrap()
    }
}
