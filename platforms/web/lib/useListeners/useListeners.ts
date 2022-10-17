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

import { RefObject, useEffect, useState } from 'react';

import { ComposerModel } from '../../generated/wysiwyg';
import { isClipboardEvent, isInputEvent } from './assert';
import { handleInput, handleKeyDown, handleSelectionChange } from './event';
import {
    FormattingFunctions,
    FormattingStates,
    InputEventProcessor,
    WysiwygInputEvent,
} from '../types';
import { TestUtilities } from '../useTestCases/types';
import { FormatBlockEvent } from './types';
import { getDefaultFormattingStates } from './utils';

type State = {
    content: string | null;
    formattingStates: FormattingStates;
};

export function useListeners(
    editorRef: RefObject<HTMLElement | null>,
    modelRef: RefObject<HTMLElement | null>,
    composerModel: ComposerModel | null,
    testUtilities: TestUtilities,
    formattingFunctions: FormattingFunctions,
    inputEventProcessor?: InputEventProcessor,
) {
    const [state, setState] = useState<State>({
        content: null,
        formattingStates: getDefaultFormattingStates(),
    });

    useEffect(() => {
        const editorNode = editorRef.current;
        if (!composerModel || !editorNode) {
            return;
        }

        const _handleInput = (e: WysiwygInputEvent) => {
            const res = handleInput(
                e,
                editorNode,
                composerModel,
                modelRef.current,
                testUtilities,
                formattingFunctions,
                inputEventProcessor,
            );

            if (res) {
                setState(({ content, formattingStates }) => {
                    const newState: State = {
                        content,
                        formattingStates:
                            res.formattingStates || formattingStates,
                    };
                    if (res.content !== undefined) {
                        newState.content = res.content;
                    }

                    return newState;
                });
            }
        };

        // React uses SyntheticEvent (https://reactjs.org/docs/events.html) and
        // doesn't catch manually fired event (myNode.dispatchEvent)
        const onInput = (e: Event) => isInputEvent(e) && _handleInput(e);
        editorNode.addEventListener('input', onInput);

        const onPaste = (e: Event) => {
            if (!isClipboardEvent(e)) {
                return;
            }

            e.preventDefault();
            e.stopPropagation();

            _handleInput(e);
        };
        editorNode.addEventListener('paste', onPaste);

        const onWysiwygInput = ((e: FormatBlockEvent) => {
            _handleInput({
                inputType: e.detail.blockType,
            } as WysiwygInputEvent);
        }) as EventListener;
        editorNode.addEventListener('wysiwygInput', onWysiwygInput);

        const onKeyDown = (e: KeyboardEvent) => {
            handleKeyDown(e, editorNode);
        };
        editorNode.addEventListener('keydown', onKeyDown);

        const onSelectionChange = () => {
            const formattingStates = handleSelectionChange(
                editorNode,
                composerModel,
                testUtilities,
            );

            if (formattingStates) {
                setState(({ content }) => ({ content, formattingStates }));
            }
        };
        document.addEventListener('selectionchange', onSelectionChange);

        return () => {
            editorNode.removeEventListener('input', onInput);
            editorNode.removeEventListener('paste', onPaste);
            editorNode.removeEventListener('wysiwygInput', onWysiwygInput);
            editorNode.removeEventListener('keydown', onKeyDown);
            document.removeEventListener('selectionchange', onSelectionChange);
        };
    }, [
        editorRef,
        composerModel,
        formattingFunctions,
        modelRef,
        testUtilities,
        inputEventProcessor,
        setState,
    ]);

    return state;
}
