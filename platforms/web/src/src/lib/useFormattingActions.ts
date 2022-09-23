import { RefObject, MouseEvent as ReactMouseEvent, useMemo } from 'react';

import { BlockType } from './types';
import { sendInputEvent } from './useListeners';

export function useFormattingActions(editorRef: RefObject<HTMLElement | null>) {
    const formattingActions = useMemo(() => {
        const sendEvent = (e: ReactMouseEvent<HTMLElement, MouseEvent> | KeyboardEvent,
            inputType: InputEvent['inputType']) => editorRef.current && sendInputEvent(e, editorRef.current, inputType);

        // The formatting action like inline code doesn't have an input type
        // For this case, we send a custom event and we do not use the browser input event handling
        const sendFormatBlockEvent = (e: ReactMouseEvent<HTMLElement, MouseEvent>, blockType: BlockType) => {
            e.preventDefault();
            e.stopPropagation();
            editorRef.current?.dispatchEvent(new CustomEvent('formatBlock', { detail: { blockType } }));
        };

        return {
            bold: (e: ReactMouseEvent<HTMLElement, MouseEvent>) => sendEvent(e, 'formatBold'),
            italic: (e: ReactMouseEvent<HTMLElement, MouseEvent>) => sendEvent(e, 'formatItalic'),
            strikeThrough: (e: ReactMouseEvent<HTMLElement, MouseEvent>) => sendEvent(e, 'formatStrikeThrough'),
            underline: (e: ReactMouseEvent<HTMLElement, MouseEvent>) => sendEvent(e, 'formatUnderline'),
            undo: (e: ReactMouseEvent<HTMLElement, MouseEvent>) => sendEvent(e, 'historyUndo'),
            redo: (e: ReactMouseEvent<HTMLElement, MouseEvent>) => sendEvent(e, 'historyRedo'),
            orderedList: (e: ReactMouseEvent<HTMLElement, MouseEvent>) => sendEvent(e, 'insertOrderedList'),
            unorderedList: (e: ReactMouseEvent<HTMLElement, MouseEvent>) => sendEvent(e, 'insertUnorderedList'),
            inlineCode: (e: ReactMouseEvent<HTMLElement, MouseEvent>) => sendFormatBlockEvent(e, 'formatInlineCode'),
        };
    }, [editorRef]);

    return formattingActions;
}
