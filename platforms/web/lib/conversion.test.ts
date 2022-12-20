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
            const convertedRichText = await plainToRich(plain);
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
        const convertedRichText = await plainToRich(plainText);
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
