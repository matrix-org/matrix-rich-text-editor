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

import { RefObject, MouseEvent as ReactMouseEvent, useMemo } from 'react';

import { BlockType } from './types';
import { sendWysiwygInputEvent } from './useListeners';

export function useFormattingActions(editorRef: RefObject<HTMLElement | null>) {
    const formattingActions = useMemo(() => {
        // The formatting action like inline code doesn't have an input type
        // Safari does not keep the inputType in an input event when the input event is fired manually
        // So we send a custom event and we do not use the browser input event handling
        const sendEvent = (
            e: ReactMouseEvent<HTMLElement, MouseEvent> | KeyboardEvent,
            blockType: BlockType,
        ) =>
            editorRef.current &&
            sendWysiwygInputEvent(e, editorRef.current, blockType);

        return {
            bold: (e: ReactMouseEvent<HTMLElement, MouseEvent>) =>
                sendEvent(e, 'formatBold'),
            italic: (e: ReactMouseEvent<HTMLElement, MouseEvent>) =>
                sendEvent(e, 'formatItalic'),
            strikeThrough: (e: ReactMouseEvent<HTMLElement, MouseEvent>) =>
                sendEvent(e, 'formatStrikeThrough'),
            underline: (e: ReactMouseEvent<HTMLElement, MouseEvent>) =>
                sendEvent(e, 'formatUnderline'),
            undo: (e: ReactMouseEvent<HTMLElement, MouseEvent>) =>
                sendEvent(e, 'historyUndo'),
            redo: (e: ReactMouseEvent<HTMLElement, MouseEvent>) =>
                sendEvent(e, 'historyRedo'),
            orderedList: (e: ReactMouseEvent<HTMLElement, MouseEvent>) =>
                sendEvent(e, 'insertOrderedList'),
            unorderedList: (e: ReactMouseEvent<HTMLElement, MouseEvent>) =>
                sendEvent(e, 'insertUnorderedList'),
            inlineCode: (e: ReactMouseEvent<HTMLElement, MouseEvent>) =>
                sendEvent(e, 'formatInlineCode'),
        };
    }, [editorRef]);

    return formattingActions;
}
