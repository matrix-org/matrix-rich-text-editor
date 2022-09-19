import { RefObject, useEffect, useMemo, useRef, useState } from "react";

// rust generated bindings
// eslint-disable-next-line camelcase
import init, { ComposerModel, new_composer_model } from '../../generated/wysiwyg.js';
import { useListeners } from "./useListeners.js";
import { useTestCases } from "./useTestCases.js";

function useEditorFocus(editorRef: RefObject<HTMLElement | null>) {
    useEffect(() => {
        console.log('call', editorRef);
        if (editorRef.current) {
            console.log('focus');
            editorRef.current.focus();
        }
    }, [editorRef]);
}

function useFormattingActions(editorRef: RefObject<HTMLElement | null>) {
    const actions = useMemo(() => {
        const sendInputEvent = (inputType: InputEvent['inputType']) =>
            editorRef.current?.dispatchEvent(new InputEvent('input', { inputType }));

        return {
            bold: () => sendInputEvent('formatBold'),
        };
    }, [editorRef]);

    return actions;
}

function useComposerModel() {
    const [composerModel, setComposerModel] = useState<ComposerModel | null>(null);

    useEffect(() => {
        init().then(() => setComposerModel(new_composer_model()));
    }, []);

    return composerModel;
}

export function useWysiwyg() {
    const ref = useRef<HTMLDivElement>(null);
    const modelRef = useRef<HTMLDivElement>(null);

    const composerModel = useComposerModel();
    const { testRef, utilities: testUtilities } = useTestCases(ref, composerModel);
    useListeners(ref, modelRef, composerModel, testUtilities);
    const formattingActions = useFormattingActions(ref);
    useEditorFocus(ref);

    console.log('rerender');

    return {
        ref,
        isWysiwygReady: Boolean(composerModel),
        wysiwyg: formattingActions,
        debug: { modelRef, testRef, resetTestCase: testUtilities.onResetTestCase },
    };
}
