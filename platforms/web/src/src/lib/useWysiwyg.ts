import { MouseEvent as ReactMouseEvent, RefObject, useEffect, useMemo, useRef, useState } from "react";

// rust generated bindings
// eslint-disable-next-line camelcase
import init, { ComposerModel, new_composer_model } from '../../generated/wysiwyg.js';
import { useListeners } from "./useListeners.js";
import { useTestCases } from "./useTestCases.js";

function useEditorFocus(editorRef: RefObject<HTMLElement | null>, isAutoFocusEnabled = false) {
    useEffect(() => {
        if (isAutoFocusEnabled) {
        // TODO remove this workaround
            const id = setTimeout(() => editorRef.current?.focus(), 200);
            return () => clearInterval(id);
        }
    }, [editorRef, isAutoFocusEnabled]);
}
/*
const actions= {
    bold: 'formatBold',
    italic: 'formatItalic',
    undo: 'historyUndo',
    redo: 'historyRedo',

};

type FormattingActions = Record<keyof typeof actions, MouseEventHandler<HTMLElement>>;*/

function useFormattingActions(editorRef: RefObject<HTMLElement | null>) {
    const formattingActions = useMemo(() => {
        const sendInputEvent = (e: ReactMouseEvent<HTMLElement, MouseEvent>, inputType: InputEvent['inputType']) => {
            e.preventDefault();
            e.stopPropagation();
            editorRef.current?.dispatchEvent(new InputEvent('input', { inputType }));
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
        };
    }, [editorRef]);

    return formattingActions;
}

function useComposerModel() {
    const [composerModel, setComposerModel] = useState<ComposerModel | null>(null);

    useEffect(() => {
        init().then(() => setComposerModel(new_composer_model()));
    }, []);

    return composerModel;
}

type WysiwygProps = {
    isAutoFocusEnabled?: boolean;
};

export function useWysiwyg(wysiwygProps?: WysiwygProps) {
    const ref = useRef<HTMLDivElement>(null);
    const modelRef = useRef<HTMLDivElement>(null);

    const composerModel = useComposerModel();
    const { testRef, utilities: testUtilities } = useTestCases(ref, composerModel);
    useListeners(ref, modelRef, composerModel, testUtilities);
    const formattingActions = useFormattingActions(ref);
    useEditorFocus(ref, wysiwygProps?.isAutoFocusEnabled);

    return {
        ref,
        isWysiwygReady: Boolean(composerModel),
        wysiwyg: formattingActions,
        debug: { modelRef, testRef, resetTestCase: testUtilities.onResetTestCase },
    };
}
