import { RefObject, MouseEvent as ReactMouseEvent, useMemo } from "react";

import { BlockType } from "./types";

/*
const actions= {
    bold: 'formatBold',
    italic: 'formatItalic',
    undo: 'historyUndo',
    redo: 'historyRedo',

};

type FormattingActions = Record<keyof typeof actions, MouseEventHandler<HTMLElement>>;*/

export function useFormattingActions(editorRef: RefObject<HTMLElement | null>) {
    const formattingActions = useMemo(() => {
        const sendInputEvent = (e: ReactMouseEvent<HTMLElement, MouseEvent>, inputType: InputEvent['inputType']) => {
            e.preventDefault();
            e.stopPropagation();
            editorRef.current?.dispatchEvent(new InputEvent('input', { inputType }));
        };

        // The formatting action like inline code doesn't have an input type
        // For this case, we send a custom event and we do not use the browser input event handling
        const sendFormatBlockEvent = (e: ReactMouseEvent<HTMLElement, MouseEvent>, blockType: BlockType) => {
            e.preventDefault();
            e.stopPropagation();
            editorRef.current?.dispatchEvent(new CustomEvent('formatBlock', { detail: { blockType } }));
        };

        /*return Object.keys(actions).reduce<FormattingActions>((acc, action) => {
            acc[action] = (e: MouseEvent) => sendInputEvent(e, actions[action]);
            return acc;
        }, {} as FormattingActions);*/

        return {
            bold: (e: ReactMouseEvent<HTMLElement, MouseEvent>) => sendInputEvent(e, 'formatBold'),
            italic: (e: ReactMouseEvent<HTMLElement, MouseEvent>) => sendInputEvent(e, 'formatItalic'),
            strikeThrough: (e: ReactMouseEvent<HTMLElement, MouseEvent>) => sendInputEvent(e, 'formatStrikeThrough'),
            underline: (e: ReactMouseEvent<HTMLElement, MouseEvent>) => sendInputEvent(e, 'formatUnderline'),
            undo: (e: ReactMouseEvent<HTMLElement, MouseEvent>) => sendInputEvent(e, 'historyUndo'),
            redo: (e: ReactMouseEvent<HTMLElement, MouseEvent>) => sendInputEvent(e, 'historyRedo'),
            orderedList: (e: ReactMouseEvent<HTMLElement, MouseEvent>) => sendInputEvent(e, 'insertOrderedList'),
            unorderedList: (e: ReactMouseEvent<HTMLElement, MouseEvent>) => sendInputEvent(e, 'insertUnorderedList'),
            inlineCode: (e: ReactMouseEvent<HTMLElement, MouseEvent>) => sendFormatBlockEvent(e, 'formatInlineCode'),
        };
    }, [editorRef]);

    return formattingActions;
}
