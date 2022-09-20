import { RefObject, useEffect, useRef, useState } from "react";

// rust generated bindings
// eslint-disable-next-line camelcase
import init, { ComposerModel, new_composer_model } from '../../generated/wysiwyg.js';
import { useFormattingActions } from "./useFormattingActions.js";
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
