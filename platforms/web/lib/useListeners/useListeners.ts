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

import { RefObject, useEffect } from 'react';

import { ComposerModel } from '../../generated/wysiwyg';
import { isInputEvent } from './assert';
import { handleInput, handleKeyDown, handleSelectionChange } from './event';
import { WysiwygInputEvent } from '../types';
import { TestUtilities } from '../useTestCases/types';
import { FormatBlockEvent } from './types';

export function useListeners(
    editorRef: RefObject<HTMLElement | null>,
    modelRef: RefObject<HTMLElement | null>,
    composerModel: ComposerModel | null,
    testUtilities: TestUtilities,
) {
    useEffect(() => {
        const editorNode = editorRef.current;
        if (!composerModel || !editorNode) {
            return;
        }

        // React uses SyntheticEvent (https://reactjs.org/docs/events.html) and doesn't catch manually fired event (myNode.dispatchEvent)
        const onInput = (e: Event) =>
            isInputEvent(e) &&
            handleInput(
                e,
                editorNode,
                composerModel,
                modelRef.current,
                testUtilities,
            );
        editorNode.addEventListener('input', onInput);

        const onWysiwygInput = ((e: FormatBlockEvent) => {
            handleInput(
                { inputType: e.detail.blockType } as WysiwygInputEvent,
                editorNode,
                composerModel,
                modelRef.current,
                testUtilities,
            );
        }) as EventListener;
        editorNode.addEventListener('wysiwygInput', onWysiwygInput);

        const onKeyDown = (e: KeyboardEvent) => {
            handleKeyDown(e, editorNode);
        };
        editorNode.addEventListener('keydown', onKeyDown);

        const onSelectionChange = () =>
            handleSelectionChange(editorNode, composerModel, testUtilities);
        document.addEventListener('selectionchange', onSelectionChange);

        return () => {
            editorNode.removeEventListener('input', onInput);
            editorNode.removeEventListener('wysiwygInput', onWysiwygInput);
            editorNode.removeEventListener('keydown', onKeyDown);
            document.removeEventListener('selectionchange', onSelectionChange);
        };
    }, [editorRef, composerModel, modelRef, testUtilities]);
}
