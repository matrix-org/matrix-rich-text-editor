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

import { richToPlain, plainToRich, markdownToPlain } from './conversion';

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
        const richText = '<p>multi</p><p>line</p>';
        const convertedPlainText = await richToPlain(richText);
        const expectedPlainText = `multi\nline`;

        expect(convertedPlainText).toBe(expectedPlainText);
    });

    it('converts linebreaks for display plain => rich', async () => {
        const plainText = 'multi\nline';
        const convertedRichText = await plainToRich(plainText, false);
        const expectedRichText = 'multi<br />line'; // TODO shouldn't the rust wrap this in <p>?

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
        // nb for correct display, there will be one \n more
        // than \\\n at the end
        const markdown = 'multiple\\\nline\\\n\\\nbreaks\\\n\\\n\\\n';
        const convertedPlainText = markdownToPlain(markdown);
        const expectedPlainText = 'multiple\nline\n\nbreaks\n\n\n\n';

        expect(convertedPlainText).toBe(expectedPlainText);
    });
});

describe('PlainToRich', () => {
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

    it('handles quotes', async () => {
        const text = '> quote from html';
        const div = document.createElement('div');
        div.innerText = text;

        const input = div.innerText;
        const expected = '<blockquote><p>quote from html</p></blockquote>';
        const output = await plainToRich(input, true);

        expect(output).toBe(expected);
    });

    it('handles some random taglike input', async () => {
        const text = '< > << >> < hi!';
        const div = document.createElement('div');
        div.innerText = text;

        const input = div.innerText;
        const expected = '&lt; &gt; &lt;&lt; &gt;&gt; &lt; hi!';
        const output = await plainToRich(input, true);

        expect(output).toBe(expected);
    });

    it('does what is expected with an angle bracket', async () => {
        const input = '> quote from string';
        const expected = '<blockquote><p>quote from string</p></blockquote>';
        const output = await plainToRich(input, true);

        expect(output).toBe(expected);
    });

    it('does not crash with html like input', async () => {
        const div = document.createElement('div');
        const text = document.createTextNode('<h1>crash!</h1>');
        div.appendChild(text);

        const input = div.innerHTML;
        const expected = '&lt;h1&gt;crash!&lt;/h1&gt;';
        const output = await plainToRich(input, true);

        expect(output).toBe(expected);
    });
});
