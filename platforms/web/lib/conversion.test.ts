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
    plainToMarkdown,
    markdownToPlain,
    amendInnerHtmlButBetter,
} from './conversion';

describe('Rich text <=> plain text', () => {
    const testCases = [
        { rich: '', plain: '' },
        { rich: 'plain', plain: 'plain' },
        { rich: '<strong>bold</strong>', plain: '__bold__' },
        { rich: '<em>italic</em>', plain: '*italic*' },
        { rich: '<u>underline</u>', plain: '<u>underline</u>' },
        { rich: '<del>strike</del>', plain: '~~strike~~' },
    ];
    const mappedTestCases = testCases.map(({ rich, plain }) => [rich, plain]);

    test.each(mappedTestCases)(
        'rich: `%s` - plain: `%s`',
        async (rich, plain) => {
            const convertedRichText = await plainToRich(plain, false);
            const convertedPlainText = await richToPlain(rich);

            expect(convertedRichText).toBe(rich);
            expect(convertedPlainText).toBe(plain);
        },
    );

    it('converts linebreaks for display rich => plain', async () => {
        const richText = 'multi<br />line';
        const convertedPlainText = await richToPlain(richText);
        const expectedPlainText = `multi\nline`;

        expect(convertedPlainText).toBe(expectedPlainText);
    });

    it('converts linebreaks for display plain => rich', async () => {
        const plainText = 'multi\nline';
        const convertedRichText = await plainToRich(plainText, false);
        const expectedRichText = 'multi<br />line';

        expect(convertedRichText).toBe(expectedRichText);
    });
});

describe('Plain text <=> markdown', () => {
    it('converts single linebreak for plain => markdown', () => {
        const plain = 'multi\nline';
        const convertedMarkdown = plainToMarkdown(plain);
        const expectedMarkdown = `multi<br />line`;

        expect(convertedMarkdown).toBe(expectedMarkdown);
    });

    it('converts multiple linebreak for plain => markdown', () => {
        // nb for correct display, there will be one br tag less
        // than \n at the end
        const plain = 'multiple\nline\n\nbreaks\n\n\n';
        const convertedMarkdown = plainToMarkdown(plain);
        const expectedMarkdown =
            'multiple<br />line<br /><br />breaks<br /><br />';

        expect(convertedMarkdown).toBe(expectedMarkdown);
    });

    it('converts single linebreak for markdown => plain', () => {
        const markdown = 'multi\\\nline';
        const convertedPlainText = markdownToPlain(markdown);
        const expectedPlainText = 'multi\nline';

        expect(convertedPlainText).toBe(expectedPlainText);
    });

    it('converts multiple linebreak for markdown => plain', () => {
        // nb for correct display, there will be one \n more
        // than \\\n at the end
        const markdown = 'multiple\\\nline\\\n\\\nbreaks\\\n\\\n\\\n';
        const convertedPlainText = markdownToPlain(markdown);
        const expectedPlainText = 'multiple\nline\n\nbreaks\n\n\n\n';

        expect(convertedPlainText).toBe(expectedPlainText);
    });
});

describe('Mentions', () => {
    it('converts at-room mentions for composer as expected', async () => {
        const input = '@room';
        const asComposerHtml = await plainToRich(input, false);

        expect(asComposerHtml).toBe(
            '<a data-mention-type="at-room" href="#" contenteditable="false">@room</a>',
        );
    });

    it('converts at-room mentions for message as expected', async () => {
        const input = '@room';
        const asMessageHtml = await plainToRich(input, true);

        expect(asMessageHtml).toBe('@room');
    });

    it('converts user mentions for composer as expected', async () => {
        const input =
            '<a href="https://matrix.to/#/@test_user:element.io" contenteditable="false" data-mention-type="user" style="some styling">a test user</a> ';
        const asComposerHtml = await plainToRich(input, false);

        expect(asComposerHtml).toMatchInlineSnapshot(
            '"<a style=\\"some styling\\" data-mention-type=\\"user\\" href=\\"https://matrix.to/#/@test_user:element.io\\" contenteditable=\\"false\\">a test user</a> "',
        );
    });

    it('converts user mentions for message as expected', async () => {
        const input =
            '<a href="https://matrix.to/#/@test_user:element.io" contenteditable="false" data-mention-type="user" style="some styling">a test user</a> ';
        const asMessageHtml = await plainToRich(input, true);

        expect(asMessageHtml).toMatchInlineSnapshot(
            '"<a href=\\"https://matrix.to/#/@test_user:element.io\\">a test user</a> "',
        );
    });

    it('converts room mentions for composer as expected', async () => {
        const input =
            '<a href="https://matrix.to/#/#test_room:element.io" contenteditable="false" data-mention-type="user" style="some styling">a test user</a> ';
        const asComposerHtml = await plainToRich(input, false);

        // note inner text is the same as the input inner text
        expect(asComposerHtml).toMatchInlineSnapshot(
            '"<a style=\\"some styling\\" data-mention-type=\\"room\\" href=\\"https://matrix.to/#/#test_room:element.io\\" contenteditable=\\"false\\">a test user</a> "',
        );
    });

    it('converts room mentions for message as expected', async () => {
        const input =
            '<a href="https://matrix.to/#/#test_room:element.io" contenteditable="false" data-mention-type="user" style="some styling">a test user</a> ';
        const asMessageHtml = await plainToRich(input, true);

        // note inner text is the mx id
        expect(asMessageHtml).toMatchInlineSnapshot(
            '"<a href=\\"https://matrix.to/#/#test_room:element.io\\">#test_room:element.io</a> "',
        );
    });
});

describe('amendHtmlInABetterWay', () => {
    let mockComposer: HTMLDivElement;
    beforeEach(() => {
        mockComposer = document.createElement('div');
    });

    it('can cope with divs with a line break', () => {
        const innerDiv = document.createElement('div');
        const innerBreak = document.createElement('br');
        innerDiv.appendChild(innerBreak);
        mockComposer.appendChild(innerDiv);

        const expected = '\n';
        expect(amendInnerHtmlButBetter(mockComposer)).toBe(expected);
    });

    it('can cope with divs with text content', () => {
        const innerDiv = document.createElement('div');
        innerDiv.appendChild(document.createTextNode('some text'));
        mockComposer.appendChild(innerDiv);

        const expected = 'some text';
        expect(amendInnerHtmlButBetter(mockComposer)).toBe(expected);
    });

    it('can cope with multiple divs with text content', () => {
        const firstInnerDiv = document.createElement('div');
        const secondInnerDiv = document.createElement('div');
        firstInnerDiv.appendChild(document.createTextNode('some text'));
        secondInnerDiv.appendChild(document.createTextNode('some more text'));

        mockComposer.append(firstInnerDiv, secondInnerDiv);

        const expected = 'some text\nsome more text';
        expect(amendInnerHtmlButBetter(mockComposer)).toBe(expected);
    });

    it('can cope div following plain text node', () => {
        const firstTextNode = 'textnode text';
        const secondDiv = document.createElement('div');
        secondDiv.appendChild(document.createTextNode('some more text'));

        mockComposer.append(firstTextNode, secondDiv);

        const expected = 'textnode text\nsome more text';
        expect(amendInnerHtmlButBetter(mockComposer)).toBe(expected);
    });

    it('can cope with multiple adjacent text nodes at top level', () => {
        const strings = ['first string', 'second string', 'third string'];
        strings.forEach((s) =>
            mockComposer.appendChild(document.createTextNode(s)),
        );

        const expected = strings.join('\n');
        expect(amendInnerHtmlButBetter(mockComposer)).toBe(expected);
    });

    it('can cope with multiple adjacent text nodes in nested div', () => {
        const innerDiv = document.createElement('div');
        const strings = ['first string', 'second string', 'third string'];
        strings.forEach((s) =>
            innerDiv.appendChild(document.createTextNode(s)),
        );
        mockComposer.appendChild(innerDiv);

        const expected = strings.join('\n');
        expect(amendInnerHtmlButBetter(mockComposer)).toBe(expected);
    });

    it('can cope with a mention at the top level', () => {
        const mention = document.createElement('a');
        mention.appendChild(document.createTextNode('inner text'));
        mention.setAttribute('href', 'testHref');
        mention.setAttribute('data-mention-type', 'testType');
        mention.setAttribute('style', 'testStyle');
        mention.setAttribute('contenteditable', 'false');
        mockComposer.appendChild(mention);

        const expected = `<a href="testHref" data-mention-type="testType" style="testStyle" contenteditable="false">inner text</a>`;
        expect(amendInnerHtmlButBetter(mockComposer)).toBe(expected);
    });

    it('can cope with a mention at the top level inline with textnodes', () => {
        const mention = document.createElement('a');
        mention.appendChild(document.createTextNode('inner text'));
        mention.setAttribute('href', 'testHref');
        mention.setAttribute('data-mention-type', 'testType');
        mention.setAttribute('style', 'testStyle');
        mention.setAttribute('contenteditable', 'false');

        mockComposer.appendChild(document.createTextNode('preceding '));
        mockComposer.appendChild(mention);
        mockComposer.appendChild(document.createTextNode(' following'));

        const expected = `preceding <a href="testHref" data-mention-type="testType" style="testStyle" contenteditable="false">inner text</a> following`;
        expect(amendInnerHtmlButBetter(mockComposer)).toBe(expected);
    });

    it('can cope with a nested mention', () => {
        const innerDiv = document.createElement('div');
        const mention = document.createElement('a');
        mention.appendChild(document.createTextNode('inner text'));
        mention.setAttribute('href', 'testHref');
        mention.setAttribute('data-mention-type', 'testType');
        mention.setAttribute('style', 'testStyle');
        mention.setAttribute('contenteditable', 'false');
        innerDiv.appendChild(mention);
        mockComposer.appendChild(innerDiv);

        const expected = `<a href="testHref" data-mention-type="testType" style="testStyle" contenteditable="false">inner text</a>`;
        expect(amendInnerHtmlButBetter(mockComposer)).toBe(expected);
    });

    it('can cope with a nested mention with nested text nodes', () => {
        const innerDiv = document.createElement('div');
        const mention = document.createElement('a');
        mention.appendChild(document.createTextNode('inner text'));
        mention.setAttribute('href', 'testHref');
        mention.setAttribute('data-mention-type', 'testType');
        mention.setAttribute('style', 'testStyle');
        mention.setAttribute('contenteditable', 'false');

        innerDiv.appendChild(document.createTextNode('preceding '));
        innerDiv.appendChild(mention);
        innerDiv.appendChild(document.createTextNode(' following'));
        mockComposer.appendChild(innerDiv);

        const expected = `preceding <a href="testHref" data-mention-type="testType" style="testStyle" contenteditable="false">inner text</a> following`;
        expect(amendInnerHtmlButBetter(mockComposer)).toBe(expected);
    });

    it('can cope with a nested mention next to top level text nodes', () => {
        const innerDiv = document.createElement('div');
        const mention = document.createElement('a');
        mention.appendChild(document.createTextNode('inner text'));
        mention.setAttribute('href', 'testHref');
        mention.setAttribute('data-mention-type', 'testType');
        mention.setAttribute('style', 'testStyle');
        mention.setAttribute('contenteditable', 'false');

        mockComposer.appendChild(document.createTextNode('preceding'));
        innerDiv.appendChild(mention);
        mockComposer.appendChild(innerDiv);
        mockComposer.appendChild(document.createTextNode('following'));

        const expected = `preceding\n<a href="testHref" data-mention-type="testType" style="testStyle" contenteditable="false">inner text</a>\nfollowing`;
        expect(amendInnerHtmlButBetter(mockComposer)).toBe(expected);
    });

    it('can cope with adjacent top level mentions', () => {
        ['1', '2', '3'].forEach((id) => {
            const mention = document.createElement('a');
            mention.appendChild(document.createTextNode('inner text' + id));
            mention.setAttribute('href', 'testHref' + id);
            mention.setAttribute('data-mention-type', 'testType' + id);
            mention.setAttribute('style', 'testStyle' + id);
            mention.setAttribute('contenteditable', 'false');

            mockComposer.appendChild(mention);
        });

        const expected = `<a href="testHref1" data-mention-type="testType1" style="testStyle1" contenteditable="false">inner text1</a><a href="testHref2" data-mention-type="testType2" style="testStyle2" contenteditable="false">inner text2</a><a href="testHref3" data-mention-type="testType3" style="testStyle3" contenteditable="false">inner text3</a>`;
        expect(amendInnerHtmlButBetter(mockComposer)).toBe(expected);
    });

    it('can cope with adjacent nested mentions', () => {
        ['1', '2', '3'].forEach((id) => {
            const mention = document.createElement('a');
            mention.appendChild(document.createTextNode('inner text' + id));
            mention.setAttribute('href', 'testHref' + id);
            mention.setAttribute('data-mention-type', 'testType' + id);
            mention.setAttribute('style', 'testStyle' + id);
            mention.setAttribute('contenteditable', 'false');

            if (id === '2') {
                const innerDiv = document.createElement('div');
                innerDiv.appendChild(mention);
                mockComposer.appendChild(innerDiv);
            } else {
                mockComposer.appendChild(mention);
            }
        });

        const expected = `<a href="testHref1" data-mention-type="testType1" style="testStyle1" contenteditable="false">inner text1</a>\n<a href="testHref2" data-mention-type="testType2" style="testStyle2" contenteditable="false">inner text2</a>\n<a href="testHref3" data-mention-type="testType3" style="testStyle3" contenteditable="false">inner text3</a>`;
        expect(amendInnerHtmlButBetter(mockComposer)).toBe(expected);
    });
});
