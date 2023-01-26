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
    act,
    fireEvent,
    render,
    screen,
    waitFor,
} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { createRef, MutableRefObject } from 'react';

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

        it(
            `Should be ${expectedActivationState} ` + `after single activation`,
            async () => {
                // When
                await act(() => userEvent.click(button));

                // Then
                await waitFor(() =>
                    expect(button).toHaveAttribute(
                        'data-state',
                        expectedActivationState,
                    ),
                );
            },
        );

        it('Should be formatted after typing', async () => {
            // When
            await act(() => userEvent.click(button));
            // Do not use userEvent.type
            // The generated inputEvent has missing attributes
            fireEvent.input(textbox, {
                data: 'foo',
                inputType: 'insertText',
            });

            // Then
            await waitFor(() =>
                expect(textbox).toContainHTML(expectedSimpleFormatting),
            );
        });

        it('Should format the selection', async () => {
            // When
            fireEvent.input(textbox, {
                data: 'foo bar',
                inputType: 'insertText',
            });
            select(textbox, 2, 4);
            await act(() => userEvent.click(button));

            // Then
            await waitFor(() =>
                expect(textbox).toContainHTML(expectedAdvancedFormatting),
            );
        });

        it('Should format the selection on multiple lines', async () => {
            // When
            await act(() =>
                userEvent.click(
                    screen.getByRole('button', { name: 'strikeThrough' }),
                ),
            );
            fireEvent.input(textbox, {
                data: 'foo',
                inputType: 'insertText',
            });
            await act(() => userEvent.type(textbox, '{enter}'));
            fireEvent.input(textbox, {
                data: 'bar',
                inputType: 'insertText',
            });

            select(textbox, 2, 5);
            await act(() => userEvent.click(button));

            // Then
            await waitFor(() =>
                expect(textbox).toContainHTML(expectedMultipleLineFormatting),
            );
        });

        it('Should unformat the selection', async () => {
            // When
            await act(() => userEvent.click(button));
            fireEvent.input(textbox, {
                data: 'foo bar',
                inputType: 'insertText',
            });
            select(textbox, 2, 4);
            await act(() => userEvent.click(button));

            // Then
            await waitFor(() => {
                expect(button).toHaveAttribute('data-state', defaultState);
                expect(textbox).toContainHTML(expectedUnformatting);
            });
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
        await act(() => userEvent.click(button));

        // Then
        await waitFor(() => expect(textbox).toContainHTML('add new words'));
    });

    it('Should insert the text when ', async () => {
        // When
        fireEvent.input(textbox, {
            data: 'foo bar',
            inputType: 'insertText',
        });
        await act(() => userEvent.click(button));

        // Then
        await waitFor(() =>
            expect(textbox).toContainHTML('foo baradd new words'),
        );
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
        await waitFor(() =>
            expect(textbox).toContainHTML(
                '<a href="https://mylink.com">my text</a>',
            ),
        );
    });

    it('Should transform the selected text into link', async () => {
        // When
        const textbox = await renderEditor('foobar');
        select(textbox, 0, 6);
        await userEvent.click(screen.getByRole('button', { name: 'link' }));

        // Then
        await waitFor(() =>
            expect(textbox).toContainHTML(
                '<a href="https://mylink.com">foobar</a>',
            ),
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
        await waitFor(() => expect(textbox).toContainHTML('foobar'));
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
        await waitFor(() =>
            expect(actionsRef.current?.getLink()).toBe('https://mylink.com'),
        );
    });
});
