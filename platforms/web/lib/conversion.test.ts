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

import {
    richToPlain,
    plainToRich,
    markdownToPlain,
    plainTextInnerHtmlToMarkdown,
} from './conversion';

describe('Rich text <=> plain text', () => {
    const testCases = [
        { rich: '', plain: '' },
        { rich: 'plain', plain: 'plain' },
        { rich: '<strong>bold</strong>', plain: '__bold__' },
        { rich: '<em>italic</em>', plain: '*italic*' },
        { rich: '<del>strike</del>', plain: '~~strike~~' },
    ];
    const mappedTestCases = testCases.map(({ rich, plain }) => [rich, plain]);

    test.each(mappedTestCases)(
        'rich: `%s` - plain: `%s`',
        async (rich, plain) => {
            const convertedRichText = await plainToRich(plain, false);
            const convertedPlainText = await richToPlain(rich, false);

            expect(convertedRichText).toBe(rich);
            expect(convertedPlainText).toBe(plain);
        },
    );

    it('converts underline case rich => plain', async () => {
        // This is the html representation of underlining
        const rich = '<u>underline</u>';

        // When we convert the plain text, we expect the output to be the `rich` string - it
        // is then set as `.innerText` in element web so that handles html escaping entities
        const expectedPlainText = '<u>underline</u>';

        const convertedPlainText = await richToPlain(rich, false);
        expect(convertedPlainText).toBe(expectedPlainText);
    });

    it('converts underline case plain => rich', async () => {
        // When the above is typed by a user in the plain text editor, the innerHTML
        // will look like this
        const plain = '&lt;u&gt;underline&lt;/u&gt;';

        const expectedRichText = '<u>underline</u>';

        const convertedRichText = await plainToRich(plain, false);
        expect(convertedRichText).toBe(expectedRichText);
    });

    it('converts linebreaks for display rich => plain', async () => {
        const richText = 'multi<br />line';
        const convertedPlainText = await richToPlain(richText, false);
        const expectedPlainText = `multi\nline`;

        expect(convertedPlainText).toBe(expectedPlainText);
    });

    it('converts blockquotes from plain => rich', async () => {
        const plainText = '> I am a quote';
        const convertedRichText = await plainToRich(plainText, false);
        const expectedRichText = '<blockquote><p>I am a quote</p></blockquote>';

        expect(convertedRichText).toBe(expectedRichText);
    });

    it('converts code spans from plain => rich', async () => {
        const plainText = '```I am a code span```';
        const convertedRichText = await plainToRich(plainText, false);
        const expectedRichText = '<code>I am a code span</code>';

        expect(convertedRichText).toBe(expectedRichText);
    });

    it('converts code blocks from plain => rich with newline separation', async () => {
        const plainText = '```\nI am a code block\n```';
        const convertedRichText = await plainToRich(plainText, false);
        const expectedRichText = '<pre><code>I am a code block</code></pre>';

        expect(convertedRichText).toBe(expectedRichText);
    });

    it('converts code blocks from plain => rich with div separation', async () => {
        const plainText = '```<div>I codeblock</div><div>```</div>';
        const convertedRichText = await plainToRich(plainText, false);
        const expectedRichText = '<pre><code>I codeblock</code></pre>';

        expect(convertedRichText).toBe(expectedRichText);
    });
});

describe('markdownToPlain', () => {
    it('converts single linebreak for markdown => plain', () => {
        const markdown = 'multi\\\nline';
        const convertedPlainText = markdownToPlain(markdown);
        const expectedPlainText = 'multi\nline';

        expect(convertedPlainText).toBe(expectedPlainText);
    });

    it('converts multiple linebreak for markdown => plain', () => {
        const markdown = 'multiple\\\nline\\\n\\\nbreaks\\\n\\\n\\\n';
        const convertedPlainText = markdownToPlain(markdown);
        const expectedPlainText = 'multiple\nline\n\nbreaks\n\n';

        expect(convertedPlainText).toBe(expectedPlainText);
    });
});

describe('Mentions', () => {
    describe('Plain => Rich', () => {
        it('converts at-room mentions for composer', async () => {
            const input = '@room';
            const asComposerHtml = await plainToRich(input, false);

            expect(asComposerHtml).toBe(
                '<a data-mention-type="at-room" href="#" contenteditable="false">@room</a>',
            );
        });

        it('converts at-room mentions for message', async () => {
            const input = '@room';
            const asMessageHtml = await plainToRich(input, true);

            expect(asMessageHtml).toBe('@room');
        });

        it('converts user mentions for composer', async () => {
            const input =
                '<a href="https://matrix.to/#/@test_user:element.io" contenteditable="false" data-mention-type="user" style="some styling">a test user</a> ';
            const asComposerHtml = await plainToRich(input, false);

            expect(asComposerHtml).toMatchInlineSnapshot(
                '"<a style=\\"some styling\\" data-mention-type=\\"user\\" href=\\"https://matrix.to/#/@test_user:element.io\\" contenteditable=\\"false\\">a test user</a> "',
            );
        });

        it('converts user mentions for message', async () => {
            const input =
                '<a href="https://matrix.to/#/@test_user:element.io" contenteditable="false" data-mention-type="user" style="some styling">a test user</a> ';
            const asMessageHtml = await plainToRich(input, true);

            expect(asMessageHtml).toMatchInlineSnapshot(
                '"<a href=\\"https://matrix.to/#/@test_user:element.io\\">a test user</a> "',
            );
        });

        it('converts room mentions for composer', async () => {
            const input =
                '<a href="https://matrix.to/#/#test_room:element.io" contenteditable="false" data-mention-type="user" style="some styling">a test user</a> ';
            const asComposerHtml = await plainToRich(input, false);

            // note inner text is the same as the input inner text
            expect(asComposerHtml).toMatchInlineSnapshot(
                '"<a style=\\"some styling\\" data-mention-type=\\"room\\" href=\\"https://matrix.to/#/#test_room:element.io\\" contenteditable=\\"false\\">a test user</a> "',
            );
        });

        it('converts room mentions for message', async () => {
            const input =
                '<a href="https://matrix.to/#/#test_room:element.io" contenteditable="false" data-mention-type="user" style="some styling">a test user</a> ';
            const asMessageHtml = await plainToRich(input, true);

            // note inner text is the mx id
            expect(asMessageHtml).toMatchInlineSnapshot(
                '"<a href=\\"https://matrix.to/#/#test_room:element.io\\">#test_room:element.io</a> "',
            );
        });
    });

    describe('Rich => Plain', () => {
        it('converts at-room mentions to html for composer', async () => {
            const input = '@room';
            const asComposerMarkdown = await richToPlain(input, false);

            expect(asComposerMarkdown).toMatchInlineSnapshot(
                '"<a data-mention-type=\\"at-room\\" href=\\"#\\" contenteditable=\\"false\\">@room</a>"',
            );
        });

        it('keeps at-room mentions as plain text for message', async () => {
            const input = '@room';
            const asMessageMarkdown = await richToPlain(input, true);

            expect(asMessageMarkdown).toBe('@room');
        });

        it('keeps user mentions as html for composer', async () => {
            const input =
                '<a href="https://matrix.to/#/@test_user:element.io">a test user</a> ';
            const asComposerMarkdown = await richToPlain(input, false);

            expect(asComposerMarkdown).toMatchInlineSnapshot(
                '"<a data-mention-type=\\"user\\" href=\\"https://matrix.to/#/@test_user:element.io\\" contenteditable=\\"false\\">a test user</a> "',
            );
        });

        it('converts user mentions to inner text for message', async () => {
            const input =
                '<a href="https://matrix.to/#/@test_user:element.io">a test user</a> ';
            const asMessageMarkdown = await richToPlain(input, true);

            expect(asMessageMarkdown).toBe('a test user ');
        });

        it('keeps room mentions as html for composer', async () => {
            const input =
                '<a href="https://matrix.to/#/#test_room:element.io">a test user</a> ';
            const asComposerMarkdown = await richToPlain(input, false);

            expect(asComposerMarkdown).toMatchInlineSnapshot(
                '"<a data-mention-type=\\"room\\" href=\\"https://matrix.to/#/#test_room:element.io\\" contenteditable=\\"false\\">a test user</a> "',
            );
        });

        it('converts room mentions to mxId for message', async () => {
            const input =
                '<a href="https://matrix.to/#/#test_room:element.io">a test user</a> ';
            const asMessageMarkdown = await richToPlain(input, true);

            // note inner text is the mx id
            expect(asMessageMarkdown).toBe('#test_room:element.io ');
        });
    });
});

// Although a bit clunky, all of these tests simulate a plain text composer by creating a content editable
// div, appending children to it and then reading the composer's innerHTML. This way we can ensure that we
// are giving the conversion function decent input for the tests.
describe('plainTextInnerHtmlToMarkdown', () => {
    let mockComposer: HTMLDivElement;

    function createMentionElement(identifier = ''): HTMLAnchorElement {
        const mention = document.createElement('a');
        mention.appendChild(document.createTextNode(`inner text${identifier}`));
        mention.setAttribute('href', `testHref${identifier}`);
        mention.setAttribute('data-mention-type', `testType${identifier}`);
        mention.setAttribute('style', `testStyle${identifier}`);
        mention.setAttribute('contenteditable', 'false');
        return mention;
    }

    function createPlaceholderDiv(): HTMLDivElement {
        const div = document.createElement('div');
        const br = document.createElement('br');
        div.appendChild(br);
        return div;
    }

    beforeEach(() => {
        mockComposer = document.createElement('div');
        mockComposer.setAttribute('contenteditable', 'true');
    });

    it('handles two lines of text, second line empty, with newline char', () => {
        const textNode = document.createTextNode('firstline\n\n');
        mockComposer.appendChild(textNode);

        const expected = 'firstline\n';
        expect(plainTextInnerHtmlToMarkdown(mockComposer.innerHTML)).toBe(
            expected,
        );
    });

    it('handles two lines of text, second line empty, with placeholder div', () => {
        const textNode = document.createTextNode('firstline');

        mockComposer.append(textNode, createPlaceholderDiv());

        const expected = 'firstline\n';
        expect(plainTextInnerHtmlToMarkdown(mockComposer.innerHTML)).toBe(
            expected,
        );
    });

    it('handles consecutive newlines between text lines for newline chars', () => {
        const textNode = document.createTextNode('first\n\n\n\nlast');
        mockComposer.appendChild(textNode);

        const expected = 'first\n\n\n\nlast';
        expect(plainTextInnerHtmlToMarkdown(mockComposer.innerHTML)).toBe(
            expected,
        );
    });

    it('handles consecutive newlines between text lines for placeholder divs', () => {
        const firstText = document.createTextNode('first');

        // after the placeholders have started, text can only be inserted inside divs
        const lastDiv = document.createElement('div');
        const lastText = document.createTextNode('last');
        lastDiv.appendChild(lastText);

        const children = [
            firstText,
            createPlaceholderDiv(),
            createPlaceholderDiv(),
            createPlaceholderDiv(),
            lastDiv,
        ];

        mockComposer.append(...children);

        const expected = 'first\n\n\n\nlast';
        expect(plainTextInnerHtmlToMarkdown(mockComposer.innerHTML)).toBe(
            expected,
        );
    });

    it('handles divs with a line break', () => {
        const innerDiv = document.createElement('div');
        const innerBreak = document.createElement('br');
        innerDiv.appendChild(innerBreak);
        mockComposer.appendChild(innerDiv);

        const expected = '\n';
        expect(plainTextInnerHtmlToMarkdown(mockComposer.innerHTML)).toBe(
            expected,
        );
    });

    it('handles divs with text content', () => {
        const innerDiv = document.createElement('div');
        innerDiv.appendChild(document.createTextNode('some text'));
        mockComposer.appendChild(innerDiv);

        const expected = 'some text';
        expect(plainTextInnerHtmlToMarkdown(mockComposer.innerHTML)).toBe(
            expected,
        );
    });

    it('handles multiple divs with text content', () => {
        const firstInnerDiv = document.createElement('div');
        const secondInnerDiv = document.createElement('div');
        firstInnerDiv.appendChild(document.createTextNode('some text'));
        secondInnerDiv.appendChild(document.createTextNode('some more text'));

        mockComposer.append(firstInnerDiv, secondInnerDiv);

        const expected = 'some text\nsome more text';
        expect(plainTextInnerHtmlToMarkdown(mockComposer.innerHTML)).toBe(
            expected,
        );
    });

    it('handles div following plain text node', () => {
        const firstTextNode = 'textnode text';
        const secondDiv = document.createElement('div');
        secondDiv.appendChild(document.createTextNode('some more text'));

        mockComposer.append(firstTextNode, secondDiv);

        const expected = 'textnode text\nsome more text';
        expect(plainTextInnerHtmlToMarkdown(mockComposer.innerHTML)).toBe(
            expected,
        );
    });

    it('handles multiple adjacent text nodes at top level', () => {
        // this is how chrome structures the child nodes
        const strings = [
            'first string',
            '\n',
            'second string',
            '\n',
            'third string',
        ];
        strings.forEach((s) =>
            mockComposer.appendChild(document.createTextNode(s)),
        );

        const expected = 'first string\nsecond string\nthird string';

        expect(plainTextInnerHtmlToMarkdown(mockComposer.innerHTML)).toBe(
            expected,
        );
    });

    it('handles multiple adjacent text nodes in nested div', () => {
        const innerDiv = document.createElement('div');
        // this is how chrome structures the child nodes
        const strings = [
            'first string',
            '\n',
            'second string',
            '\n',
            'third string',
        ];
        strings.forEach((s) =>
            innerDiv.appendChild(document.createTextNode(s)),
        );
        mockComposer.appendChild(innerDiv);

        const expected = 'first string\nsecond string\nthird string';
        expect(plainTextInnerHtmlToMarkdown(mockComposer.innerHTML)).toBe(
            expected,
        );
    });

    it('handles a mention at the top level', () => {
        const mention = createMentionElement();
        mockComposer.appendChild(mention);

        // eslint-disable-next-line max-len
        const expected = `<a href="testHref" data-mention-type="testType" style="testStyle" contenteditable="false">inner text</a>`;
        expect(plainTextInnerHtmlToMarkdown(mockComposer.innerHTML)).toBe(
            expected,
        );
    });

    it('handles a mention at the top level inline with textnodes', () => {
        const mention = createMentionElement();

        mockComposer.appendChild(document.createTextNode('preceding '));
        mockComposer.appendChild(mention);
        mockComposer.appendChild(document.createTextNode(' following'));

        // eslint-disable-next-line max-len
        const expected = `preceding <a href="testHref" data-mention-type="testType" style="testStyle" contenteditable="false">inner text</a> following`;
        expect(plainTextInnerHtmlToMarkdown(mockComposer.innerHTML)).toBe(
            expected,
        );
    });

    it('handles a nested mention', () => {
        const innerDiv = document.createElement('div');
        const mention = createMentionElement();
        innerDiv.appendChild(mention);
        mockComposer.appendChild(innerDiv);

        // eslint-disable-next-line max-len
        const expected = `<a href="testHref" data-mention-type="testType" style="testStyle" contenteditable="false">inner text</a>`;
        expect(plainTextInnerHtmlToMarkdown(mockComposer.innerHTML)).toBe(
            expected,
        );
    });

    it('handles a nested mention with nested text nodes', () => {
        const innerDiv = document.createElement('div');
        const mention = createMentionElement();

        innerDiv.appendChild(document.createTextNode('preceding '));
        innerDiv.appendChild(mention);
        innerDiv.appendChild(document.createTextNode(' following'));
        mockComposer.appendChild(innerDiv);

        // eslint-disable-next-line max-len
        const expected = `preceding <a href="testHref" data-mention-type="testType" style="testStyle" contenteditable="false">inner text</a> following`;
        expect(plainTextInnerHtmlToMarkdown(mockComposer.innerHTML)).toBe(
            expected,
        );
    });

    it('handles a nested mention next to top level text node', () => {
        const innerDiv = document.createElement('div');
        const mention = createMentionElement();

        mockComposer.appendChild(document.createTextNode('preceding'));
        innerDiv.appendChild(mention);
        mockComposer.appendChild(innerDiv);

        // eslint-disable-next-line max-len
        const expected = `preceding\n<a href="testHref" data-mention-type="testType" style="testStyle" contenteditable="false">inner text</a>`;
        expect(plainTextInnerHtmlToMarkdown(mockComposer.innerHTML)).toBe(
            expected,
        );
    });

    it('handles adjacent top level mentions', () => {
        ['1', '2', '3'].forEach((id) => {
            const mention = createMentionElement(id);
            mockComposer.appendChild(mention);
        });

        // eslint-disable-next-line max-len
        const expected = `<a href="testHref1" data-mention-type="testType1" style="testStyle1" contenteditable="false">inner text1</a><a href="testHref2" data-mention-type="testType2" style="testStyle2" contenteditable="false">inner text2</a><a href="testHref3" data-mention-type="testType3" style="testStyle3" contenteditable="false">inner text3</a>`;
        expect(plainTextInnerHtmlToMarkdown(mockComposer.innerHTML)).toBe(
            expected,
        );
    });

    it('handles adjacent nested mentions', () => {
        const innerDiv = document.createElement('div');
        ['1', '2', '3'].forEach((id) => {
            const mention = createMentionElement(id);
            innerDiv.appendChild(mention);
        });
        mockComposer.appendChild(innerDiv);

        // eslint-disable-next-line max-len
        const expected = `<a href="testHref1" data-mention-type="testType1" style="testStyle1" contenteditable="false">inner text1</a><a href="testHref2" data-mention-type="testType2" style="testStyle2" contenteditable="false">inner text2</a><a href="testHref3" data-mention-type="testType3" style="testStyle3" contenteditable="false">inner text3</a>`;
        expect(plainTextInnerHtmlToMarkdown(mockComposer.innerHTML)).toBe(
            expected,
        );
    });

    it('handles adjacent top level and nested mentions', () => {
        const topLevelMention = createMentionElement('1');
        const nestedMention = createMentionElement('2');

        const innerDiv = document.createElement('div');
        innerDiv.appendChild(nestedMention);

        mockComposer.append(topLevelMention, innerDiv);

        // eslint-disable-next-line max-len
        const expected = `<a href="testHref1" data-mention-type="testType1" style="testStyle1" contenteditable="false">inner text1</a>\n<a href="testHref2" data-mention-type="testType2" style="testStyle2" contenteditable="false">inner text2</a>`;
        expect(plainTextInnerHtmlToMarkdown(mockComposer.innerHTML)).toBe(
            expected,
        );
    });

    it('outputs to console.debug for unexpected node types', () => {
        const debugSpy = vi
            .spyOn(console, 'debug')
            .mockImplementation(() => {});

        const unorderedList = document.createElement('ul');
        const listItem = document.createElement('li');
        const textNode = document.createTextNode('list text');

        listItem.appendChild(textNode);
        unorderedList.appendChild(listItem);
        mockComposer.appendChild(unorderedList);

        // check that it pulls the inner text out correctly, despite the unexpected tags
        const expected = `list text`;
        expect(plainTextInnerHtmlToMarkdown(mockComposer.innerHTML)).toBe(
            expected,
        );

        // check that it has also called console.debug with some useful output
        expect(debugSpy.mock.calls).toMatchInlineSnapshot(`
          [
            [
              "Converting unexpected node type UL",
            ],
            [
              "Converting unexpected node type LI",
            ],
          ]
        `);

        // reset the spy
        debugSpy.mockRestore();
    });
});
