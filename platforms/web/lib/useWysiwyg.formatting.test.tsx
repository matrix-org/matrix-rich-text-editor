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

import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { createRef, MutableRefObject } from 'react';
import { preview } from 'vite';

import { Editor } from './testUtils/Editor';
import { select } from './testUtils/selection';
import { FormattingFunctions } from './types';

describe.each([
    [
        'bold',
        'enabled',
        'reversed',
        '<strong>foo</strong>',
        'fo<strong>o&nbsp;</strong>bar',
        // eslint-disable-next-line max-len
        '<p><del>fo<strong>o</strong></del></p><p><del><strong>b</strong>ar</del></p>',
        '<strong>fo</strong>o <strong>bar</strong>',
    ],
    [
        'italic',
        'enabled',
        'reversed',
        '<em>foo</em>',
        'fo<em>o&nbsp;</em>bar',
        '<p><del>fo<em>o</em></del></p><p><del><em>b</em>ar</del></p>',
        '<em>fo</em>o <em>bar</em>',
    ],
    [
        'underline',
        'enabled',
        'reversed',
        '<u>foo</u>',
        'fo<u>o&nbsp;</u>bar',
        '<p><del>fo<u>o</u></del></p><p><del><u>b</u>ar</del></p>',
        '<u>fo</u>o <u>bar</u>',
    ],
    [
        'strikeThrough',
        'enabled',
        'reversed',
        '<del>foo</del>',
        'fo<del>o&nbsp;</del>bar',
        '<p><del>fo</del>o</p><p>b<del>ar</del></p>',
        '<del>fo</del>o <del>bar</del>',
    ],
    [
        'orderedList',
        'enabled',
        'reversed',
        '<ol><li>foo</li></ol>',
        '<ol><li>foo bar</li></ol>',
        '<ol><li><del>foo</del></li><li><del>bar</del></li></ol>',
        'foo bar',
    ],
    [
        'unorderedList',
        'enabled',
        'reversed',
        '<ul><li>foo</li></ul>',
        '<ul><li>foo bar</li></ul>',
        '<ul><li><del>foo</del></li><li><del>bar</del></li></ul>',
        'foo bar',
    ],
    [
        'inlineCode',
        'enabled',
        'reversed',
        '<code>foo</code>',
        'fo<code>o&nbsp;</code>bar',
        '<p><del>fo</del><code>o</code></p><p><code>b</code><del>ar</del></p>',
        '<code>fo</code>o <code>bar</code>',
    ],
    [
        'codeBlock',
        'enabled',
        'reversed',
        '<pre><code>foo</code></pre>',
        '<pre><code>foo bar</code></pre>',
        '<pre><code><del>foo</del>\n<del>bar</del></code></pre>',
        'foo bar',
    ],
    [
        'quote',
        'enabled',
        'reversed',
        '<blockquote><p>foo</p></blockquote>',
        '<blockquote><p>foo bar</p></blockquote>',
        '<blockquote><p><del>foo</del></p><p><del>bar</del></p></blockquote>',
        'foo bar',
    ],
])(
    'Formatting %s',
    (
        name,
        defaultState,
        expectedActivationState,
        expectedSimpleFormatting,
        expectedAdvancedFormatting,
        expectedMultipleLineFormatting,
        expectedUnformatting,
    ) => {
        let button: HTMLButtonElement;
        let textbox: HTMLDivElement;

        beforeEach(async () => {
            render(<Editor />);
            textbox = screen.getByRole('textbox');
            await waitFor(() =>
                expect(textbox).toHaveAttribute('contentEditable', 'true'),
            );
            button = screen.getByRole('button', { name });
        });

        it(`Should be ${defaultState} by default`, async () => {
            // Then
            expect(button).toHaveAttribute('data-state', defaultState);
        });

        // eslint-disable-next-line max-len
        it(`Should be ${expectedActivationState} after single activation`, async () => {
            // When
            await userEvent.click(button);

            // Then
            expect(button).toHaveAttribute(
                'data-state',
                expectedActivationState,
            );
        });

        it('Should be formatted after typing', async () => {
            // When
            await userEvent.click(button);
            // Do not use userEvent.type
            // The generated inputEvent has missing attributes
            fireEvent.input(textbox, {
                data: 'foo',
                inputType: 'insertText',
            });

            // Then
            expect(textbox).toContainHTML(expectedSimpleFormatting);
        });

        it('Should format the selection', async () => {
            // When
            fireEvent.input(textbox, {
                data: 'foo bar',
                inputType: 'insertText',
            });
            select(textbox, 2, 4);
            await userEvent.click(button);

            // Then
            expect(textbox).toContainHTML(expectedAdvancedFormatting);
        });

        it('Should format the selection on multiple lines', async () => {
            // When
            await userEvent.click(
                screen.getByRole('button', { name: 'strikeThrough' }),
            );
            fireEvent.input(textbox, {
                data: 'foo',
                inputType: 'insertText',
            });
            await userEvent.type(textbox, '{enter}');
            fireEvent.input(textbox, {
                data: 'bar',
                inputType: 'insertText',
            });

            select(textbox, 2, 5);
            await userEvent.click(button);

            // Then
            expect(textbox).toContainHTML(expectedMultipleLineFormatting);
        });

        it('Should unformat the selection', async () => {
            // When
            await userEvent.click(button);
            fireEvent.input(textbox, {
                data: 'foo bar',
                inputType: 'insertText',
            });
            select(textbox, 2, 4);
            await userEvent.click(button);

            // Then
            expect(button).toHaveAttribute('data-state', defaultState);
            expect(textbox).toContainHTML(expectedUnformatting);
        });
    },
);

describe('insertText', () => {
    let button: HTMLButtonElement;
    let textbox: HTMLDivElement;

    beforeEach(async () => {
        render(<Editor />);
        textbox = screen.getByRole('textbox');
        await waitFor(() =>
            expect(textbox).toHaveAttribute('contentEditable', 'true'),
        );
        button = screen.getByRole('button', { name: 'insertText' });
    });

    it('Should insert the text when empty', async () => {
        // When
        await userEvent.click(button);

        // Then
        expect(textbox).toContainHTML('add new words');
    });

    it('Should insert the text when ', async () => {
        // When
        fireEvent.input(textbox, {
            data: 'foo bar',
            inputType: 'insertText',
        });
        await userEvent.click(button);

        // Then
        expect(textbox).toContainHTML('foo baradd new words');
    });
});

describe('link', () => {
    async function renderEditor(
        initialContent?: string,
        ref?: MutableRefObject<FormattingFunctions | null>,
    ) {
        render(<Editor initialContent={initialContent} actionsRef={ref} />);
        const textbox: HTMLDivElement = screen.getByRole('textbox');
        await waitFor(() =>
            expect(textbox).toHaveAttribute('contentEditable', 'true'),
        );
        return textbox;
    }

    it('Should insert the link with text', async () => {
        // When
        const textbox = await renderEditor();
        await userEvent.click(
            screen.getByRole('button', {
                name: 'link with text',
            }),
        );

        // Then
        expect(textbox).toContainHTML(
            '<a href="https://mylink.com">my text</a>',
        );
    });

    it('Should transform the selected text into link', async () => {
        // When
        const textbox = await renderEditor('foobar');
        select(textbox, 0, 6);
        await userEvent.click(screen.getByRole('button', { name: 'link' }));

        // Then
        expect(textbox).toContainHTML(
            '<a href="https://mylink.com">foobar</a>',
        );
    });

    it('Should remove the link', async () => {
        // When
        const textbox = await renderEditor(
            '<a href="https://mylink.com">foobar</a>',
        );
        select(textbox, 0, 6);
        await userEvent.click(
            screen.getByRole('button', { name: 'remove links' }),
        );

        // Then
        expect(textbox).toContainHTML('foobar');
    });

    it('Should get the link', async () => {
        // When
        const actionsRef = createRef<FormattingFunctions>();
        const textbox = await renderEditor(
            '<a href="https://mylink.com">foobar</a>',
            actionsRef,
        );
        select(textbox, 0, 6);

        // Then
        expect(actionsRef.current?.getLink()).toBe('https://mylink.com');
    });
});

describe('indentation', () => {
    let textbox: HTMLDivElement;

    beforeEach(async () => {
        render(<Editor />);
        textbox = screen.getByRole('textbox');
        await waitFor(() =>
            expect(textbox).toHaveAttribute('contentEditable', 'true'),
        );
    });

    it('Should not show the indent/unindent buttons when empty', async () => {
        const indentButton = screen.queryByRole('button', { name: 'indent' });
        const unindentButton = screen.queryByRole('button', {
            name: 'unindent',
        });

        [indentButton, unindentButton].forEach((button) => {
            expect(button).not.toBeInTheDocument();
        });
    });

    // eslint-disable-next-line max-len
    it('Should show the indent/unindent buttons when a list is reversed', async () => {
        await userEvent.click(
            screen.getByRole('button', { name: 'orderedList' }),
        );

        const indentButton = screen.getByRole('button', {
            name: 'indent',
        });
        const unindentButton = screen.getByRole('button', {
            name: 'unindent',
        });

        [indentButton, unindentButton].forEach((button) => {
            expect(button).toBeInTheDocument();
        });
    });

    // eslint-disable-next-line max-len
    it('Should not be able to change indentation on first list item', async () => {
        await userEvent.click(
            screen.getByRole('button', { name: 'orderedList' }),
        );
        await userEvent.type(textbox, 'foo');

        const indentButton = screen.getByRole('button', {
            name: 'indent',
        });

        expect(indentButton).toHaveAttribute('data-state', 'disabled');
    });

    // eslint-disable-next-line max-len
    it('Should be able to change indentation on second list item', async () => {
        // Select the ordered list and then enter two lines of input
        await userEvent.click(
            screen.getByRole('button', { name: 'orderedList' }),
        );

        fireEvent.input(textbox, {
            data: 'foo',
            inputType: 'insertText',
        });
        await userEvent.type(textbox, '{enter}');
        fireEvent.input(textbox, {
            data: 'bar',
            inputType: 'insertText',
        });

        const indentButton = screen.getByRole('button', {
            name: 'indent',
        });

        // check that the indent button is enabled and we have a single list
        expect(indentButton).toHaveAttribute('data-state', 'enabled');
        expect(screen.getAllByRole('list')).toHaveLength(1);

        // click the button and then check that we have two lists (as we nest
        // lists to implement indentation)
        await userEvent.click(indentButton);
        expect(screen.getAllByRole('list')).toHaveLength(2);

        // now reverse the actions
        const unindentButton = screen.getByRole('button', {
            name: 'unindent',
        });
        expect(unindentButton).toHaveAttribute('data-state', 'enabled');
        await userEvent.click(unindentButton);
        expect(screen.getAllByRole('list')).toHaveLength(1);
    });
});

describe('mentions', () => {
    let button: HTMLButtonElement;
    let textbox: HTMLDivElement;

    beforeEach(async () => {
        render(<Editor />);
        textbox = screen.getByRole('textbox');
        button = screen.getByRole('button', { name: 'add @mention' });
        await waitFor(() =>
            expect(textbox).toHaveAttribute('contentEditable', 'true'),
        );
    });

    it('does not add a mention on click with an incorrect prefix', async () => {
        // When
        const noPrefixInput = 'noPrefix';
        fireEvent.input(textbox, {
            data: noPrefixInput,
            inputType: 'insertText',
        });
        await userEvent.click(button);

        // Then
        await expect(textbox).toContainHTML(noPrefixInput);
    });

    it.each(['@at', '#hash'])(
        'adds a mention with prefix %s',
        async (prefixedInput) => {
            // When
            fireEvent.input(textbox, {
                data: prefixedInput,
                inputType: 'insertText',
            });
            await userEvent.click(button);

            // Then
            // nb this information is hardcoded in the button for these tests so
            // they should all yield the same result
            const link = screen.getByText('test user');
            expect(link).toBeInTheDocument();
            expect(link).toHaveAttribute('contenteditable', 'false');
            screen.debug();
            expect(link).toHaveAttribute('data-mention-type');
        },
    );
});

describe('commands', () => {
    let button: HTMLButtonElement;
    let textbox: HTMLDivElement;

    beforeEach(async () => {
        render(<Editor />);
        textbox = screen.getByRole('textbox');
        button = screen.getByRole('button', { name: 'add command' });
        await waitFor(() =>
            expect(textbox).toHaveAttribute('contentEditable', 'true'),
        );
    });

    it('does not add a command on click with an incorrect prefix', async () => {
        // When
        const noPrefixInput = 'noPrefix';
        fireEvent.input(textbox, {
            data: noPrefixInput,
            inputType: 'insertText',
        });
        await userEvent.click(button);

        // Then
        await expect(textbox).toContainHTML(noPrefixInput);
    });

    it('adds a command with prefix /slash', async () => {
        const prefixedInput = '/slash';
        // When
        fireEvent.input(textbox, {
            data: prefixedInput,
            inputType: 'insertText',
        });
        await userEvent.click(button);

        // Then
        // nb this information is hardcoded in the button for this test
        expect(textbox).toContainHTML('/test_command');
    });
});
