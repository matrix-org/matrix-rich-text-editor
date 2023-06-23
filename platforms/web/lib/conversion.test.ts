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
});
