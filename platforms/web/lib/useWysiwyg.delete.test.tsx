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

import { Editor } from './testUtils/Editor';
import { select } from './testUtils/selection';

describe('delete content', () => {
    let clearButton: HTMLButtonElement;
    let textbox: HTMLDivElement;

    beforeEach(async () => {
        render(<Editor />);
        textbox = screen.getByRole('textbox');
        await waitFor(() =>
            expect(textbox).toHaveAttribute('contentEditable', 'true'),
        );
        clearButton = screen.getByRole('button', { name: 'clear' });
    });

    async function fillContent() {
        fireEvent.input(textbox, {
            data: 'foo',
            inputType: 'insertText',
        });
        await userEvent.type(textbox, '{enter}');
        fireEvent.input(textbox, {
            data: 'bar',
            inputType: 'insertText',
        });
    }

    it('Should delete the content when using clear button', async () => {
        // When
        await fillContent();
        await userEvent.click(clearButton);

        // Then
        await waitFor(() => expect(textbox).toHaveTextContent(/^$/));
    });

    it('Should delete one character when using backspace', async () => {
        // When
        await fillContent();

        select(textbox, 2, 2);
        await userEvent.type(textbox, '{backspace}');

        // Then
        await waitFor(() => {
            expect(textbox).toContainHTML('<p>fo</p><p>bar</p>');
            expect(textbox).toHaveAttribute(
                'data-content',
                '<p>fo</p><p>bar</p>',
            );
        });
    });

    it('Should delete the selection when using backspace', async () => {
        // When
        await fillContent();

        select(textbox, 2, 5);
        await userEvent.type(textbox, '{backspace}');

        // Then
        await waitFor(() => {
            expect(textbox).toContainHTML('foar');
            expect(textbox).toHaveAttribute('data-content', 'foar');
        });
    });

    it('Should delete one character when using delete', async () => {
        // When
        fireEvent.input(textbox, {
            data: 'foobar',
            inputType: 'insertText',
        });
        select(textbox, 3, 3);
        fireEvent.input(textbox, { inputType: 'deleteContentForward' });

        // Then
        await waitFor(() => {
            expect(textbox).toContainHTML('fooar');
            expect(textbox).toHaveAttribute('data-content', 'fooar');
        });
    });
});
