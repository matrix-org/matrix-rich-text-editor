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

import { Editor } from './testUtils/Editor';

describe('inputEventProcessor', () => {
    const inputEventProcessor = vitest.fn();
    let textbox: HTMLElement;

    beforeEach(async () => {
        render(<Editor inputEventProcessor={inputEventProcessor} />);
        textbox = screen.getByRole('textbox');
        await waitFor(() =>
            expect(textbox).toHaveAttribute('contentEditable', 'true'),
        );
    });

    it('Should call inputEventProcessor', async () => {
        // When
        inputEventProcessor.mockReturnValue(null);
        fireEvent.input(textbox, {
            data: 'foo',
            inputType: 'insertText',
        });

        // Then
        await waitFor(() => {
            expect(textbox).toContainHTML('');
            expect(textbox).toHaveAttribute('data-content', '');
            expect(inputEventProcessor).toBeCalledTimes(1);
            expect(inputEventProcessor).toBeCalledWith(
                new InputEvent('input', {
                    data: 'foo',
                    inputType: 'insertText',
                }),
                expect.anything(),
            );
        });
    });
});
