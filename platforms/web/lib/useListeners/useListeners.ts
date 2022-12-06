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
    AllActionStates,
    InputEventProcessor,
    WysiwygInputEvent,
} from '../types';
import { TestUtilities } from '../useTestCases/types';
import { FormatBlockEvent } from './types';
import { createDefaultActionStates, mapToAllActionStates } from './utils';

type State = {
    content: string | null;
    actionStates: AllActionStates;
};

export function useListeners(
    editorRef: RefObject<HTMLElement | null>,
    modelRef: RefObject<HTMLElement | null>,
    composerModel: ComposerModel | null,
    testUtilities: TestUtilities,
    formattingFunctions: FormattingFunctions,
    initialContent?: string,
    inputEventProcessor?: InputEventProcessor,
) {
    const [state, setState] = useState<State>({
        content: initialContent || null,
        actionStates: createDefaultActionStates(),
    });

    useEffect(() => {
        if (composerModel) {
            setState({
                content: composerModel.get_content_as_html(),
                actionStates: mapToAllActionStates(
                    composerModel.action_states(),
                ),
            });
        }
    }, [composerModel]);

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
                setState(({ content, actionStates }) => {
                    const newState: State = {
                        content,
                        actionStates: res.actionStates || actionStates,
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
                data: e.detail.data,
            } as WysiwygInputEvent);
        }) as EventListener;
        editorNode.addEventListener('wysiwygInput', onWysiwygInput);

        const onKeyDown = (e: KeyboardEvent) => {
            handleKeyDown(e, editorNode);
        };
        editorNode.addEventListener('keydown', onKeyDown);

        const onSelectionChange = () => {
            const actionStates = handleSelectionChange(
                editorNode,
                composerModel,
                testUtilities,
            );

            if (actionStates) {
                setState(({ content }) => ({
                    content,
                    actionStates,
                }));
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
