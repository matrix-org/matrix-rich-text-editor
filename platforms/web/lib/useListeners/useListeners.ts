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

import { RefObject, useEffect, useRef, useState } from 'react';

import { ComposerModel, SuggestionPattern } from '../../generated/wysiwyg';
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
    suggestion: SuggestionPattern | null;
};

export function useListeners(
    editorRef: RefObject<HTMLElement | null>,
    modelRef: RefObject<HTMLElement | null>,
    composerModel: ComposerModel | null,
    testUtilities: TestUtilities,
    formattingFunctions: FormattingFunctions,
    onError: (content?: string) => void,
    initialContent?: string,
    inputEventProcessor?: InputEventProcessor,
) {
    const [state, setState] = useState<State>({
        content: initialContent || null,
        actionStates: createDefaultActionStates(),
        suggestion: null,
    });

    const plainTextContentRef = useRef<string>();

    const [areListenersReady, setAreListenersReady] = useState(false);

    useEffect(() => {
        if (composerModel) {
            setState({
                content: composerModel.get_content_as_html(),
                actionStates: mapToAllActionStates(
                    composerModel.action_states(),
                ),
                suggestion: null,
            });
            plainTextContentRef.current =
                composerModel.get_content_as_plain_text();
        }
    }, [composerModel]);

    useEffect(() => {
        const editorNode = editorRef.current;
        if (!composerModel || !editorNode) {
            return;
        }

        const _handleInput = (e: WysiwygInputEvent) => {
            try {
                const res = handleInput(
                    e,
                    editorNode,
                    composerModel,
                    modelRef.current,
                    testUtilities,
                    formattingFunctions,
                    state.suggestion,
                    inputEventProcessor,
                );

                if (res) {
                    setState((prevState) => {
                        // the state here is different for each piece of state
                        // state.content: update it if not undefined
                        const content =
                            res.content !== undefined
                                ? res.content
                                : prevState.content;

                        // state.actionStates: update if they are non-null
                        const actionStates =
                            res.actionStates || prevState.actionStates;

                        // state.suggestion: update even if null
                        const suggestion = res.suggestion;

                        return {
                            content,
                            actionStates,
                            suggestion,
                        };
                    });
                    plainTextContentRef.current =
                        composerModel.get_content_as_plain_text();
                }
            } catch (e) {
                onError(plainTextContentRef.current);
            }
        };

        // React uses SyntheticEvent (https://reactjs.org/docs/events.html) and
        // doesn't catch manually fired event (myNode.dispatchEvent)
        const onInput = (e: Event) => isInputEvent(e) && _handleInput(e);
        editorNode.addEventListener('input', onInput);

        // Can be called by onPaste or onBeforeInput
        const onPaste = (e: ClipboardEvent | InputEvent) => {
            // this is required to handle edge case image pasting in Safari, see
            // https://github.com/vector-im/element-web/issues/25327
            const isSpecialCaseInputEvent =
                isInputEvent(e) &&
                e.inputType === 'insertFromPaste' &&
                e.dataTransfer !== null;

            const isEventToHandle =
                isClipboardEvent(e) || isSpecialCaseInputEvent;

            if (isEventToHandle) {
                e.preventDefault();
                e.stopPropagation();

                _handleInput(e);
            }
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
            handleKeyDown(
                e,
                editorNode,
                composerModel,
                formattingFunctions,
                inputEventProcessor,
            );
        };
        editorNode.addEventListener('keydown', onKeyDown);

        const onSelectionChange = () => {
            try {
                const actionStates = handleSelectionChange(
                    editorNode,
                    composerModel,
                    testUtilities,
                );

                if (actionStates) {
                    setState(({ content, suggestion }) => ({
                        content,
                        actionStates,
                        suggestion,
                    }));
                }
                plainTextContentRef.current =
                    composerModel.get_content_as_plain_text();
            } catch (e) {
                onError(plainTextContentRef.current);
            }
        };
        document.addEventListener('selectionchange', onSelectionChange);

        // this is required to handle edge case image pasting in Safari, see
        // https://github.com/vector-im/element-web/issues/25327
        const onBeforeInput = onPaste;
        editorNode.addEventListener('beforeinput', onBeforeInput);

        setAreListenersReady(true);

        return () => {
            setAreListenersReady(false);
            editorNode.removeEventListener('input', onInput);
            editorNode.removeEventListener('paste', onPaste);
            editorNode.removeEventListener('wysiwygInput', onWysiwygInput);
            editorNode.removeEventListener('keydown', onKeyDown);
            editorNode.removeEventListener('beforeinput', onBeforeInput);
            document.removeEventListener('selectionchange', onSelectionChange);
        };
    }, [
        editorRef,
        composerModel,
        formattingFunctions,
        modelRef,
        testUtilities,
        inputEventProcessor,
        onError,
        plainTextContentRef,
        state.suggestion,
    ]);

    return { areListenersReady, ...state };
}
