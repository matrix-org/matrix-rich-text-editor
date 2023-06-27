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
// export const plainToMarkdown = (plainText: string) => {
//     let markdown = plainText;
//     if (markdown.endsWith('\n')) {
//         // manually remove the final linebreak
//         markdown = markdown.slice(0, -1);
//     }
//     return markdown.replaceAll(/\n/g, '<br />');
// };

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

export async function plainToRich(plainText: string, inMessageFormat: boolean) {
    if (plainText.length === 0) {
        return '';
    }

    // this function could be called before initialising the WASM
    // so we need to try to initialise
    await initOnce();

    // convert the plain text into markdown so that we can use it to
    // set the model
    const markdown = plainTextInnerHtmlToMarkdown(plainText);

    // set the model and return the rich text
    const model = new_composer_model();
    model.set_content_from_markdown(markdown);

    return inMessageFormat
        ? model.get_content_as_message_html()
        : model.get_content_as_html();
}

/*
The reason for requiring this function requires it's own explanation, so here it is.
When manipulating a content editable div in a browser, as we do for the plain text version
of the composer in element web, there is a limited subset of html that the composer can contain.
Currently, the innerHTML of the plain text composer can only contain:
  - text with `\n` line separation if `shift + enter` is used to insert the linebreak
    - in this case, inserting a newline after a single word will result in `word\n\n`, then
      subsequent typing will replace the final `\n` to give `word\nanother word`
  - text with <div> separation if `cmd + enter to send` is enabled and `enter` is used to insert
    the linebreak
    - in this case, inserting a newline inserts `<div><br></div>`, and then subsequent typing 
      replaces the <br> tag with the new content
  - mentions (ie <a> tags with special attributes) which can be at the top level, or nested inside
    a div 
What we need to do is to get this input into a good shape for the markdown parser in the rust model. 
Because of some of the intricacies of how text content is parsed when you use `.innerHTML` vs `.innerText`
we do it manually so that we can extract:
  - text content from any text nodes exactly as the user has written it, so that there is no escaping
    of html entities like < or &
  - mentions in their pure html form so that they can be passed through as valid html, as the mentions
    in the plain text composer can be parsed into mentions inside the rust model
*/

const NEWLINE_CHAR = '\n';
export function plainTextInnerHtmlToMarkdown(innerHtml: string): string {
    // Parse the innerHtml into a DOM and treat the `body` as the `composer
    const { body: composer } = new DOMParser().parseFromString(
        innerHtml,
        'text/html',
    );

    // When we parse the nodes, we need to manually add newlines if the node is either
    // adjacent to a div or is the last child and it's parent is adjacent to a div
    function shouldAddNewlineCharacter(node: Node): boolean {
        const nextSibling = node.nextSibling || node.parentElement?.nextSibling;

        if (!nextSibling) return false;

        return nextSibling?.nodeName === 'DIV';
    }

    // Create an iterator to allow us to traverse the DOM node by node
    const i = document.createNodeIterator(composer, NodeFilter.SHOW_ALL);
    let node = i.nextNode();

    // Use this to store the manually built markdown output
    let markdownOutput = '';

    while (node !== null) {
        // TEXT NODES - `node` represents the text node, only handle if not inside a mention
        const isTextNodeToHandle =
            node.nodeName === '#text' && node.parentElement?.nodeName !== 'A';

        // MENTION NODES - `node` represents the enclosing <a> tag
        const isMentionToHandle = node.nodeName === 'A';

        // LINEBREAK DIVS - `node` represents the enclosing <div> tag
        const isDivContainingBreak =
            node.nodeName === 'DIV' &&
            node.childNodes.length === 1 &&
            node.firstChild?.nodeName === 'BR';

        if (isDivContainingBreak) {
            markdownOutput += NEWLINE_CHAR;
        } else if (isTextNodeToHandle) {
            // content is the text itself, unescaped i.e. > is >, not &gt;
            let content = node.textContent;
            if (shouldAddNewlineCharacter(node)) {
                content += NEWLINE_CHAR;
            }
            markdownOutput += content;
        } else if (isMentionToHandle) {
            // content is the html of the mention i.e. <a ...attributes>text</a>
            let content = node.firstChild?.parentElement?.outerHTML ?? '';
            if (shouldAddNewlineCharacter(node)) {
                content += NEWLINE_CHAR;
            }
            markdownOutput += content;
        }

        node = i.nextNode();
    }

    // After converting the DOM, we need to trim a single `\n` off the end of the
    // output if we have consecutive newlines, as this is a browser placeholder
    if (markdownOutput.endsWith(NEWLINE_CHAR.repeat(2))) {
        markdownOutput = markdownOutput.slice(0, -1);
    }

    return markdownOutput;
}
