/*
Copyright 2022 The Matrix.org Foundation C.I.C.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

// rust generated bindings
import {
    // eslint-disable-next-line camelcase
    new_composer_model,
} from '../generated/wysiwyg.js';

// const backslash = String.fromCharCode(0x005c);
export function richToPlain(richText: string) {
    if (richText.length === 0) {
        return '';
    }
    const model = new_composer_model();
    model.set_content_from_html(richText);

    // in plain text, newlines will be represented as \n for display
    // as the display box can not interpret markdown
    const markdown = model.get_content_as_markdown();
    const plainText = markdown.replaceAll(/\\/g, '');

    return plainText;
}

export function plainToRich(plainText: string) {
    if (plainText.length === 0) {
        return '';
    }
    // in plain text, newlines will be represented as \n if input
    // by the user pressing enter, so we want to convert these to
    // valid markdown for parsing by MarkdownHTMLParser::to_html
    const markdown = plainText.replaceAll(/\n/g, '<br />\n');
    const model = new_composer_model();
    model.set_content_from_markdown(markdown);
    const richText = model.get_content_as_html();

    return richText;
}
