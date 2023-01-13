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

import { ComposerModel } from '../generated/wysiwyg';
import { processInput } from './composer';
import { FormattingFunctions } from './types';

const mockComposerModel = {
    replace_text: vi.fn(),
    code_block: vi.fn(),
} as unknown as ComposerModel;

const mockAction = vi.fn();

const mockFormattingFunctions = {} as unknown as FormattingFunctions;

function inpEv(inputType: string, data?: string): InputEvent {
    return new InputEvent('InputEvent', { data, inputType });
}

describe('processInput', () => {
    beforeEach(() => {
        vi.resetAllMocks();
    });

    it('returns early if inputEventProcessor returns null', () => {
        const mockInputEventProcessor = vi.fn().mockReturnValue(null);

        processInput(
            new InputEvent('some event'),
            mockComposerModel,
            mockAction,
            mockFormattingFunctions,
            mockInputEventProcessor,
        );

        expect(mockAction).not.toHaveBeenCalled();
    });

    it('handles clipboard events with replace_text', () => {
        const clipboardContent = 'clipboard data';
        const e = new ClipboardEvent('some clipboard event');
        const mockGetter = vi.fn().mockReturnValue(clipboardContent);

        // We can't easily generate the correct type here, so disable ts
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        e.clipboardData = { getData: mockGetter };

        processInput(e, mockComposerModel, mockAction, mockFormattingFunctions);

        expect(mockGetter).toHaveBeenCalledTimes(1);
        expect(mockComposerModel.replace_text).toHaveBeenCalledWith(
            clipboardContent,
        );
        expect(mockAction).toHaveBeenCalledWith(undefined, 'paste');
    });

    it('handles falsy clipboard events with replace_text', () => {
        const clipboardContent = null;
        const e = new ClipboardEvent('some clipboard event');
        const mockGetter = vi.fn().mockReturnValue(clipboardContent);

        // We can't easily generate the correct type here, so disable ts
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        e.clipboardData = { getData: mockGetter };

        processInput(e, mockComposerModel, mockAction, mockFormattingFunctions);

        expect(mockGetter).toHaveBeenCalledTimes(1);
        expect(mockComposerModel.replace_text).toHaveBeenCalledWith('');
        expect(mockAction).toHaveBeenCalledWith(undefined, 'paste');
    });

    it('handles insertText with replace_text', () => {
        const e = inpEv('insertText', 'goo');

        // When we process the input
        processInput(e, mockComposerModel, mockAction, mockFormattingFunctions);

        // Then replace_text and mockAction have been called correctly;
        expect(mockComposerModel.replace_text).toHaveBeenCalledWith('goo');
        expect(mockAction).toHaveBeenCalledWith(
            undefined,
            'replace_text',
            'goo',
        );
    });

    it('handles insertCompositionText with replace_text', () => {
        const e = inpEv('insertCompositionText', 'gar');

        // When we process the input
        processInput(e, mockComposerModel, mockAction, mockFormattingFunctions);

        // Then replace_text and mockAction have been called correctly;
        expect(mockComposerModel.replace_text).toHaveBeenCalledWith('gar');
        expect(mockAction).toHaveBeenCalledWith(
            undefined,
            'replace_text',
            'gar',
        );
    });

    it('handles insertFromComposition with replace_text', () => {
        const e = inpEv('insertFromComposition', 'gaz');

        // When we process the input
        processInput(e, mockComposerModel, mockAction, mockFormattingFunctions);

        // Then replace_text and mockAction have been called correctly;
        expect(mockComposerModel.replace_text).toHaveBeenCalledWith('gaz');
        expect(mockAction).toHaveBeenCalledWith(
            undefined,
            'replace_text',
            'gaz',
        );
    });

    it('handles insertCodeBlock with code_block', () => {
        const e = new InputEvent('insertCodeBlock', {
            inputType: 'insertCodeBlock',
        });

        // When we process the input
        processInput(e, mockComposerModel, mockAction, mockFormattingFunctions);

        // Then code_block and mockAction have been called correctly;
        expect(mockComposerModel.code_block).toHaveBeenCalledTimes(1);
        expect(mockAction).toHaveBeenCalledWith(undefined, 'code_block');
    });
});
