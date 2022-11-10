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
import { act } from 'react-dom/test-utils';

import { Editor } from './testUtils/Editor';

describe('Undo redo', () => {
    let undo: HTMLButtonElement;
    let redo: HTMLButtonElement;
    let textbox: HTMLDivElement;

    beforeEach(async () => {
        render(<Editor />);
        textbox = screen.getByRole('textbox');
        await waitFor(() =>
            expect(textbox).toHaveAttribute('contentEditable', 'true'),
        );
        undo = screen.getByRole('button', { name: 'undo' });
        redo = screen.getByRole('button', { name: 'redo' });
    });

    test('Should be disabled by default', () => {
        // Then
        expect(undo).toHaveAttribute('data-state', 'disabled');
        expect(redo).toHaveAttribute('data-state', 'disabled');
    });

    test('Should undo and redo content', async () => {
        // When
        fireEvent.input(textbox, {
            data: 'foo bar',
            inputType: 'insertText',
        });

        // Then
        expect(undo).toHaveAttribute('data-state', 'enabled');
        expect(redo).toHaveAttribute('data-state', 'disabled');

        // When
        await act(() => userEvent.click(undo));

        // Then
        expect(textbox).toHaveTextContent(/^$/);
        expect(undo).toHaveAttribute('data-state', 'disabled');
        expect(redo).toHaveAttribute('data-state', 'enabled');

        // When
        await act(() => userEvent.click(redo));

        // Then
        expect(textbox).toHaveTextContent(/^foo bar$/);
        expect(undo).toHaveAttribute('data-state', 'enabled');
        expect(redo).toHaveAttribute('data-state', 'disabled');
    });
});
