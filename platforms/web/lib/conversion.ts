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
      subsequent typing will replace the final `\n` to give `word\nother word`
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

export function plainTextInnerHtmlToMarkdown(composer: HTMLDivElement): string {
    const i = document.createNodeIterator(composer, NodeFilter.SHOW_ALL);
    let node = i.nextNode();
    // loop through every single node and do...something
    let outputStuff = '';
    while (node !== null) {
        const isTopLevelTextNode =
            node.nodeName === '#text' &&
            node.parentElement?.isSameNode(composer);
        const isNestedTextNode =
            node.nodeName === '#text' &&
            !node.parentElement?.isSameNode(composer) &&
            node.parentElement?.nodeName === 'DIV';
        const isTopLevelMention =
            node.nodeName === '#text' &&
            node.parentElement?.nodeName === 'A' &&
            node.parentElement?.parentElement?.isSameNode(composer);
        const isNestedMention =
            node.nodeName === '#text' &&
            node.parentElement?.nodeName === 'A' &&
            node.parentElement?.parentElement?.nodeName === 'DIV' &&
            !node.parentElement.parentElement.isSameNode(composer);
        const isLineBreak =
            node.nodeName === 'BR' && node.parentElement?.nodeName === 'DIV';

        // if we find a br inside a div, take an \n
        if (isLineBreak) {
            outputStuff += '\n';
        }

        // if we find a text node inside a nested div, take the text content
        if (isNestedTextNode) {
            let content = node.textContent;
            const nextSibling =
                node.nextSibling || node.parentElement?.nextSibling;
            if (nextSibling && nextSibling.nodeName !== 'A') {
                content += '\n';
            }
            outputStuff += content;
        }

        // if we find a top level text node, take the text content
        if (isTopLevelTextNode) {
            let content = node.textContent;
            if (
                node.nextSibling !== null &&
                node.nextSibling.nodeName !== 'A'
            ) {
                content += '\n';
            }
            outputStuff += content;
        }

        // for a top level mention, grab the outerHTML
        if (isTopLevelMention) {
            outputStuff += node.parentElement?.outerHTML;
            const nextSibling = node.parentElement?.nextSibling;
            const isNextToBlockNode =
                nextSibling && !['#text', 'A'].includes(nextSibling.nodeName);
            if (isNextToBlockNode) {
                outputStuff += '\n';
            }
        }

        // for a nested mention, grab the outerHTML but we need to consider if we add a newline or not
        if (isNestedMention) {
            outputStuff += node.parentElement?.outerHTML;
            const isNextToBlockNode =
                node.parentElement?.nextSibling !== null &&
                !['#text', 'A'].includes(
                    node.parentElement?.nextSibling.nodeName ?? '',
                );
            const isInDivNextToAnything =
                node.parentElement?.nextSibling === null &&
                node.parentElement?.parentElement?.nextSibling !== null;
            if (isInDivNextToAnything || isNextToBlockNode) {
                outputStuff += '\n';
            }
        }

        node = i.nextNode();
    }

    return outputStuff;
}
