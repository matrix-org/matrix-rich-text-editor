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

import { ComposerModel, ComposerUpdate } from '../generated/wysiwyg';
import { processInput } from './composer';
import { FormattingFunctions } from './types';

class FakeComposerModel {
    // eslint-disable-next-line camelcase
    replace_text(newText: string): ComposerUpdate {
        replacedWithText = newText;
        return composerUpdateReplaceText;
    }
}

const formattingFunctions: FormattingFunctions = {
    bold: () => {},
    italic: () => {},
    strikeThrough: () => {},
    underline: () => {},
    undo: () => {},
    redo: () => {},
    orderedList: () => {},
    unorderedList: () => {},
    inlineCode: () => {},
    clear: () => {},
    insertText: (text: string) => {},
};

let replacedWithText: string | null = null;
const composerUpdateReplaceText = new ComposerUpdate();
const composerModel = new FakeComposerModel() as unknown as ComposerModel;
const action = (update: ComposerUpdate | null) => update;

function inpEv(inputType: string, data: string): InputEvent {
    return new InputEvent('InputEvent', { data, inputType });
}

describe('processInput', () => {
    it('handles insertText with replace_text', () => {
        const e = inpEv('insertText', 'goo');

        // When we send this type of input
        const actual = processInput(
            e,
            composerModel,
            action,
            formattingFunctions,
        );

        // Then we get the expected update out
        expect(actual).toBe(composerUpdateReplaceText);
        expect(replacedWithText).toBe('goo');
    });

    it('handles insertCompositionText with replace_text', () => {
        const e = inpEv('insertCompositionText', 'gar');

        // When we send this type of input
        const actual = processInput(
            e,
            composerModel,
            action,
            formattingFunctions,
        );

        // Then we get the expected update out
        expect(actual).toBe(composerUpdateReplaceText);
        expect(replacedWithText).toBe('gar');
    });

    it('handles insertFromComposition with replace_text', () => {
        const e = inpEv('insertFromComposition', 'gaz');

        // When we send this type of input
        const actual = processInput(
            e,
            composerModel,
            action,
            formattingFunctions,
        );

        // Then we get the expected update out
        expect(actual).toBe(composerUpdateReplaceText);
        expect(replacedWithText).toBe('gaz');
    });
});
