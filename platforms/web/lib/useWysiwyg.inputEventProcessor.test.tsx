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
        inputEventProcessor.mockReset();
    });

    it('Should call inputEventProcess on keydown', async () => {
        // When
        fireEvent.keyDown(textbox, { key: 'A', code: 'KeyA' });

        await waitFor(() => {
            expect(inputEventProcessor).toBeCalledTimes(1);
            expect(inputEventProcessor).toBeCalledWith(
                new KeyboardEvent('keyDown', {
                    key: 'A',
                    code: 'KeyA',
                }),
                expect.anything(),
                textbox,
            );
        });
    });

    /**
     *
     * This fails in a flaky way. I (AndyB) spent quite a bit of time today
     * trying to figure out why it's failing, and look for a workaround. The
     * situation improves if you add other interactions e.g. send a `type` user
     * event, but it still fails from time to time.
     *
     * There must be something we should be waiting for or similar but I really
     * can't figure it out. I also can't see why this test is problematic and
     * others are not.
     *
     * If the test is run on its own, it tends to pass, but if others are
     * running, it is less reliable. This leads me to suspect there is a timing
     * issue.
     */
    it.skip('Should call inputEventProcessor', async () => {
        // When
        inputEventProcessor.mockReturnValue(null);
        fireEvent.input(textbox, {
            data: 'foo',
            inputType: 'insertText',
        });

        // Then
        await waitFor(() => {
            expect(textbox).toHaveTextContent(/^$/);
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
