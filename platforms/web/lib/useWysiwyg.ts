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

import { RefObject, useEffect, useRef } from 'react';

import { InputEventProcessor } from './types.js';
import { useFormattingFunctions } from './useFormattingFunctions';
import { useComposerModel } from './useComposerModel';
import { useListeners } from './useListeners';
import { useTestCases } from './useTestCases';

function useEditorFocus(
    editorRef: RefObject<HTMLElement | null>,
    isAutoFocusEnabled = false,
) {
    useEffect(() => {
        if (isAutoFocusEnabled) {
            // TODO remove this workaround
            const id = setTimeout(() => editorRef.current?.focus(), 200);
            return () => clearTimeout(id);
        }
    }, [editorRef, isAutoFocusEnabled]);
}

function useEditor() {
    const ref = useRef<HTMLDivElement | null>(null);

    useEffect(() => {
        if (!ref.current?.childElementCount) {
            ref.current?.appendChild(document.createElement('br'));
        }
    }, [ref]);

    return ref;
}

export type WysiwygProps = {
    isAutoFocusEnabled?: boolean;
    inputEventProcessor?: InputEventProcessor;
    initialContent?: string;
};

export function useWysiwyg(wysiwygProps?: WysiwygProps) {
    const ref = useEditor();
    const modelRef = useRef<HTMLDivElement>(null);

    const composerModel = useComposerModel(ref, wysiwygProps?.initialContent);
    const { testRef, utilities: testUtilities } = useTestCases(
        ref,
        composerModel,
    );

    const formattingFunctions = useFormattingFunctions(ref);

    const { content, formattingStates } = useListeners(
        ref,
        modelRef,
        composerModel,
        testUtilities,
        formattingFunctions,
        wysiwygProps?.inputEventProcessor,
    );

    useEditorFocus(ref, wysiwygProps?.isAutoFocusEnabled);

    return {
        ref,
        isWysiwygReady: Boolean(composerModel),
        wysiwyg: formattingFunctions,
        content: content || wysiwygProps?.initialContent,
        formattingStates,
        debug: {
            modelRef,
            testRef,
            resetTestCase: testUtilities.onResetTestCase,
            traceAction: testUtilities.traceAction,
        },
    };
}
