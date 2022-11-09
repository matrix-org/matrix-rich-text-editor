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

import { Editor } from './testUtils/Editor';
import { select } from './testUtils/selection';

describe.each([
    [
        'bold',
        'enabled',
        'reversed',
        '<strong>foo</strong>',
        'fo<strong>o&nbsp;</strong>bar',
        '<del>fo<strong>o</strong><strong><br/>b</strong>ar</del>',
        '<strong>fo</strong>o <strong>bar</strong>',
    ],
    [
        'italic',
        'enabled',
        'reversed',
        '<em>foo</em>',
        'fo<em>o&nbsp;</em>bar',
        '<del>fo<em>o</em><em><br/>b</em>ar</del>',
        '<em>fo</em>o <em>bar</em>',
    ],
    [
        'underline',
        'enabled',
        'reversed',
        '<u>foo</u>',
        'fo<u>o&nbsp;</u>bar',
        '<del>fo<u>o</u><u><br/>b</u>ar</del>',
        '<u>fo</u>o <u>bar</u>',
    ],
    [
        'strikethrough',
        'enabled',
        'reversed',
        '<del>foo</del>',
        'fo<del>o&nbsp;</del>bar',
        '<del>fo</del>o<br/>b<del>ar</del>',
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
                    screen.getByRole('button', { name: 'strikethrough' }),
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
