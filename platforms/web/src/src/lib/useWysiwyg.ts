import { RefObject, useEffect, useMemo, useRef, useState } from "react";

// rust bindings generated
// eslint-disable-next-line camelcase
import init, { ComposerModel, new_composer_model } from '../../generated/wysiwyg.js';
import { isInputEvent } from "./assert.js";
import { handleInput, handleSelectionChange } from "./listeners.js";

function useEditorFocus(editorRef: RefObject<HTMLElement | null>, composerModel: ComposerModel | null) {
    useEffect(() => {
        if (editorRef.current && composerModel) {
            editorRef.current.focus();
        }
    }, [editorRef, composerModel]);
}

function useActions(editorRef: RefObject<HTMLElement | null>) {
    const actions = useMemo(() => {
        const sendInputEvent = (inputType: InputEvent['inputType']) =>
            editorRef.current?.dispatchEvent(new InputEvent('input', { inputType }));

        return {
            bold: () => sendInputEvent('formatBold'),
        };
    }, [editorRef]);

    return actions;
}

function useListeners(
    editorRef: RefObject<HTMLElement | null>,
    modelRef: RefObject<HTMLElement | null>,
    composerModel: ComposerModel | null,
) {
    useEffect(() => {
        const editorNode = editorRef.current;
        if (!composerModel || !editorNode) {
            return;
        }

        // React uses SyntheticEvent (https://reactjs.org/docs/events.html) and doesn't catch manually fired event (myNode.dispatchEvent)
        const onInput = (e: Event) =>
            isInputEvent(e) && handleInput(e, editorNode, composerModel, modelRef.current);
        editorNode.addEventListener('input', onInput);

        const onSelectionChange = () => handleSelectionChange(editorNode, composerModel);
        document.addEventListener('selectionchange', onSelectionChange);

        return () => {
            editorNode.removeEventListener('input', onInput);
            document.removeEventListener('selectionchange', onSelectionChange);
        };
    }, [editorRef, composerModel, modelRef]);
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
    useListeners(ref, modelRef, composerModel);
    const actions = useActions(ref);
    useEditorFocus(ref, composerModel);

    return { ref, modelRef, isWysiwygReady: Boolean(composerModel), wysiwyg: actions };
}
