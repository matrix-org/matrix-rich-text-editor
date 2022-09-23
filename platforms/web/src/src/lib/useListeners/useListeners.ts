import { RefObject, useEffect } from 'react';

import { ComposerModel } from '../../../generated/wysiwyg';
import { isInputEvent } from '../assert';
import { handleInput, handleKeyDown, handleSelectionChange } from './event';
import { WysiwygInputEvent } from '../types';
import { TestUtilities } from '../useTestCases/types';

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
            isInputEvent(e) && handleInput(
                e,
                editorNode,
                composerModel,
                modelRef.current,
                testUtilities,
            );
        editorNode.addEventListener('input', onInput);

        const onFormatBlock = () => {
            handleInput(
                { inputType: 'formatInlineCode' } as WysiwygInputEvent,
                editorNode,
                composerModel,
                modelRef.current,
                testUtilities,
            );
        };
        editorNode.addEventListener('formatBlock', onFormatBlock);

        const onKeyDown = (e: KeyboardEvent) => {
            handleKeyDown(e, editorNode);
        };
        editorNode.addEventListener('keydown', onKeyDown);

        const onSelectionChange = () => handleSelectionChange(editorNode, composerModel, testUtilities);
        document.addEventListener('selectionchange', onSelectionChange);

        return () => {
            editorNode.removeEventListener('input', onInput);
            editorNode.removeEventListener('formatBlock', onFormatBlock);
            editorNode.removeEventListener('keydown', onKeyDown);
            document.removeEventListener('selectionchange', onSelectionChange);
        };
    }, [editorRef, composerModel, modelRef, testUtilities]);
}
