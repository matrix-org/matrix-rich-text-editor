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

import { RefObject, useMemo } from 'react';

import { BlockType, FormattingFunctions } from './types';
import { sendWysiwygInputEvent } from './useListeners';
import { LinkEvent } from './useListeners/types';

export function useFormattingFunctions(
    editorRef: RefObject<HTMLElement | null>,
) {
    const formattingFunctions = useMemo<FormattingFunctions>(() => {
        // The formatting action like inline code doesn't have an input type
        // Safari does not keep the inputType in an input event
        // when the input event is fired manually, so we send a custom event
        // and we do not use the browser input event handling
        const sendEvent = (
            blockType: BlockType,
            data?: string | LinkEvent['data'],
        ) =>
            editorRef.current &&
            sendWysiwygInputEvent(
                editorRef.current,
                blockType,
                undefined,
                data,
            );

        return {
            bold: () => sendEvent('formatBold'),
            italic: () => sendEvent('formatItalic'),
            strikeThrough: () => sendEvent('formatStrikeThrough'),
            underline: () => sendEvent('formatUnderline'),
            undo: () => sendEvent('historyUndo'),
            redo: () => sendEvent('historyRedo'),
            orderedList: () => sendEvent('insertOrderedList'),
            unorderedList: () => sendEvent('insertUnorderedList'),
            inlineCode: () => sendEvent('formatInlineCode'),
            clear: () => sendEvent('clear'),
            insertText: (text: string) => sendEvent('insertText', text),
            link: (link: string, text: string) =>
                sendEvent('insertLink', { link, text }),
        };
    }, [editorRef]);

    return formattingFunctions;
}
