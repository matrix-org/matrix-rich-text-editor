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
import { initOnce } from './useComposerModel.js';

// In plain text, due to cursor positioning, ending at a linebreak will
// include an extra \n, so trim that off if required.
// We replace the remaining \n with valid markdown before
// parsing by MarkdownHTMLParser::to_html.
export const plainToMarkdown = (plainText: string) => {
    let markdown = plainText;
    if (markdown.endsWith('\n')) {
        // manually remove the final linebreak
        markdown = markdown.slice(0, -1);
    }
    return markdown.replaceAll(/\n/g, '<br />');
};

// In plain text, markdown newlines (displays '\' character followed by
// a newline character) will be represented as \n for display as the
// display box can not interpret markdown.
// We also may need to manually add a \n to account for trailing newlines.
export const markdownToPlain = (markdown: string) => {
    let plainText = markdown;
    if (plainText.endsWith('\n')) {
        // for cursor positioning we need to manually add another linebreak
        plainText = `${plainText}\n`;
    }
    return plainText.replaceAll(/\\/g, '');
};

export async function richToPlain(richText: string) {
    if (richText.length === 0) {
        return '';
    }

    // this function could be called before initialising the WASM
    // so we need to try to initialise
    await initOnce();

    // the rich text in the web model is html so set the model with it
    const model = new_composer_model();
    model.set_content_from_html(richText);

    // transform the markdown to plain text for display
    const markdown = model.get_content_as_markdown();
    const plainText = markdownToPlain(markdown);

    return plainText;
}

export async function plainToRich(plainText: string) {
    if (plainText.length === 0) {
        return '';
    }

    // this function could be called before initialising the WASM
    // so we need to try to initialise
    await initOnce();

    // convert the plain text into markdown so that we can use it to
    // set the model
    const markdown = plainToMarkdown(plainText);

    // set the model and return the rich text
    const model = new_composer_model();
    model.set_content_from_markdown(markdown);
    const richText = model.get_content_as_message_html();

    return richText;
}
