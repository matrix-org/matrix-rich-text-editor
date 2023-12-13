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

use md_parser::Event;
use pulldown_cmark as md_parser;

use crate::{dom::MarkdownParseError, UnicodeString};

pub struct MarkdownHTMLParser {}

impl MarkdownHTMLParser {
    pub fn to_html<S>(markdown: &S) -> Result<S, MarkdownParseError>
    where
        S: UnicodeString,
    {
        use md_parser::{html::push_html as compile_to_html, Options, Parser};

        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);

        let markdown = markdown.to_string();
        let parser_events: Vec<_> = Parser::new_ext(&markdown, options)
            .map(|event| match event {
                Event::SoftBreak => Event::HardBreak,
                _ => event,
            })
            .collect();

        let mut html = String::new();

        compile_to_html(&mut html, parser_events.into_iter());

        // By default, there is a `<p>…</p>\n` around the HTML content. That's the
        // correct way to handle a text block in Markdown. But it breaks our
        // assumption regarding the HTML markup. So let's remove it.
        // write me a function that gives me the number of substrings contained in a string:

        let html = {
            if html.starts_with("<p>") && html.matches("<p>").count() == 1 {
                let p = "<p>".len();
                let ppnl = "</p>\n".len();

                html[p..html.len() - ppnl].to_string()
            } else {
                html[..].to_string()
            }
        };

        // Remove any trailing newline characters from block tags
        let html = html
            .replace("<ul>\n", "<ul>")
            .replace("</ul>\n", "</ul>")
            .replace("<ol>\n", "<ol>")
            .replace("</ol>\n", "</ol>")
            .replace("</li>\n", "</li>")
            .replace("<br />\n", "<br />")
            .replace("<blockquote>\n", "<blockquote>")
            .replace("</blockquote>\n", "</blockquote>")
            .replace("<pre>\n", "<pre>")
            .replace("</pre>\n", "</pre>")
            .replace("<p>\n", "<p>")
            .replace("</p>\n", "</p>");

        // Remove the newline from the end of the single code tag that wraps the content
        // of a formatted codeblock
        let html = html.replace("\n</code>", "</code>");

        Ok(S::try_from(html).unwrap())
    }
}
