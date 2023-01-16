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
    backspace_word: vi.fn(),
    delete_word: vi.fn(),
    delete: vi.fn(),
    inline_code: vi.fn(),
    ordered_list: vi.fn(),
    unordered_list: vi.fn(),
    enter: vi.fn(),
} as unknown as ComposerModel;

const mockAction = vi.fn();

const mockFormattingFunctions = {} as unknown as FormattingFunctions;

function inpEv(inputType: string, data?: string): InputEvent {
    return new InputEvent('InputEvent', { data, inputType });
}

const consoleErrorSpy = vi.spyOn(console, 'error');

describe('processInput', () => {
    beforeEach(() => {
        vi.resetAllMocks();
    });

    afterAll(() => {
        vi.restoreAllMocks();
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
        const e = inpEv('insertCodeBlock');

        // When we process the input
        processInput(e, mockComposerModel, mockAction, mockFormattingFunctions);

        // Then code_block and mockAction have been called correctly;
        expect(mockComposerModel.code_block).toHaveBeenCalledTimes(1);
        expect(mockAction).toHaveBeenCalledWith(undefined, 'code_block');
    });

    it('handles deleteWordBackward with backspace_word', () => {
        const e = inpEv('deleteWordBackward');

        // When we process the input
        processInput(e, mockComposerModel, mockAction, mockFormattingFunctions);

        // Then code_block and mockAction have been called correctly;
        expect(mockComposerModel.backspace_word).toHaveBeenCalledTimes(1);
        expect(mockAction).toHaveBeenCalledWith(undefined, 'backspace_word');
    });

    it('handles deleteWordForward with delete_word', () => {
        const e = inpEv('deleteWordForward');

        // When we process the input
        processInput(e, mockComposerModel, mockAction, mockFormattingFunctions);

        // Then code_block and mockAction have been called correctly;
        expect(mockComposerModel.delete_word).toHaveBeenCalledTimes(1);
        expect(mockAction).toHaveBeenCalledWith(undefined, 'delete_word');
    });

    it('handles deleteByCut with delete', () => {
        const e = inpEv('deleteByCut');

        // When we process the input
        processInput(e, mockComposerModel, mockAction, mockFormattingFunctions);

        // Then code_block and mockAction have been called correctly;
        expect(mockComposerModel.delete).toHaveBeenCalledTimes(1);
        expect(mockAction).toHaveBeenCalledWith(undefined, 'delete');
    });

    it('handles formatInlineCode with inline_code', () => {
        const e = inpEv('formatInlineCode');

        // When we process the input
        processInput(e, mockComposerModel, mockAction, mockFormattingFunctions);

        // Then code_block and mockAction have been called correctly;
        expect(mockComposerModel.inline_code).toHaveBeenCalledTimes(1);
        expect(mockAction).toHaveBeenCalledWith(undefined, 'inline_code');
    });

    it('handles insertFromPaste without calling action', () => {
        const e = inpEv('insertFromPaste');

        // When we process the input
        processInput(e, mockComposerModel, mockAction, mockFormattingFunctions);

        // Then code_block and mockAction have been called correctly;
        expect(mockAction).not.toHaveBeenCalled();
    });

    it('handles insertOrderedList with ordered_list', () => {
        const e = inpEv('insertOrderedList');

        // When we process the input
        processInput(e, mockComposerModel, mockAction, mockFormattingFunctions);

        // Then code_block and mockAction have been called correctly;
        expect(mockComposerModel.ordered_list).toHaveBeenCalledTimes(1);
        expect(mockAction).toHaveBeenCalledWith(undefined, 'ordered_list');
    });

    it('handles insertUnorderedList with unordered_list', () => {
        const e = inpEv('insertUnorderedList');

        // When we process the input
        processInput(e, mockComposerModel, mockAction, mockFormattingFunctions);

        // Then code_block and mockAction have been called correctly;
        expect(mockComposerModel.unordered_list).toHaveBeenCalledTimes(1);
        expect(mockAction).toHaveBeenCalledWith(undefined, 'unordered_list');
    });

    it('handles insertLineBreak with enter', () => {
        const e = inpEv('insertLineBreak');

        // When we process the input
        processInput(e, mockComposerModel, mockAction, mockFormattingFunctions);

        // Then code_block and mockAction have been called correctly;
        expect(mockComposerModel.enter).toHaveBeenCalledTimes(1);
        expect(mockAction).toHaveBeenCalledWith(undefined, 'enter');
    });

    it('hits the break statement in insert text if input data is falsy', () => {
        const e = inpEv('insertText', '');

        // When we process the input
        processInput(e, mockComposerModel, mockAction, mockFormattingFunctions);

        // Then code_block and mockAction have been called correctly;
        expect(mockAction).not.toHaveBeenCalled();
    });

    it('returns null from a send message event', () => {
        const e = inpEv('sendMessage');

        // When we process the input
        const returnValue = processInput(
            e,
            mockComposerModel,
            mockAction,
            mockFormattingFunctions,
        );

        // Then code_block and mockAction have been called correctly;
        expect(mockAction).not.toHaveBeenCalled();
        expect(returnValue).toBe(null);
    });

    it('errors and returns null when unknown event is used', () => {
        const eventType = 'aNewEventType';
        const e = inpEv(eventType);

        // When we process the input
        const returnValue = processInput(
            e,
            mockComposerModel,
            mockAction,
            mockFormattingFunctions,
        );

        // Then code_block and mockAction have been called correctly;
        expect(mockAction).not.toHaveBeenCalled();
        expect(consoleErrorSpy).toHaveBeenCalledWith(
            `Unknown input type: ${eventType}`,
        );
        expect(returnValue).toBe(null);
    });
});
