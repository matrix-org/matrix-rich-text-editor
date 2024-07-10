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

import { RefObject, useEffect, useMemo, useRef } from 'react';

import {
    AllActionStates,
    FormattingFunctions,
    InputEventProcessor,
    MappedSuggestion,
} from './types.js';
import { useFormattingFunctions } from './useFormattingFunctions';
import { useComposerModel } from './useComposerModel';
import { useListeners } from './useListeners';
import { useTestCases } from './useTestCases';
import { mapSuggestion } from './suggestion.js';
import { TraceAction } from './useTestCases/types.js';

export { richToPlain, plainToRich } from './conversion';

function useEditorFocus(
    editorRef: RefObject<HTMLElement | null>,
    isAutoFocusEnabled = false,
): void {
    useEffect(() => {
        if (isAutoFocusEnabled) {
            // TODO remove this workaround
            const id = setTimeout(() => editorRef.current?.focus(), 200);
            return (): void => clearTimeout(id);
        }
    }, [editorRef, isAutoFocusEnabled]);
}

function useEditor(): React.MutableRefObject<HTMLDivElement | null> {
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

export type UseWysiwyg = {
    ref: React.MutableRefObject<HTMLDivElement | null>;
    isWysiwygReady: boolean;
    wysiwyg: FormattingFunctions;
    content: string | null;
    actionStates: AllActionStates;
    debug: {
        modelRef: RefObject<HTMLDivElement>;
        testRef: RefObject<HTMLDivElement>;
        resetTestCase: () => void | null;
        traceAction: TraceAction;
    };
    suggestion: MappedSuggestion | null;
    messageContent: string | null;
};

export function useWysiwyg(wysiwygProps?: WysiwygProps): UseWysiwyg {
    const ref = useEditor();
    const modelRef = useRef<HTMLDivElement>(null);

    const { composerModel, onError } = useComposerModel(
        ref,
        wysiwygProps?.initialContent,
    );
    const { testRef, utilities: testUtilities } = useTestCases(
        ref,
        composerModel,
    );

    const formattingFunctions = useFormattingFunctions(ref, composerModel);

    const { content, actionStates, areListenersReady, suggestion } =
        useListeners(
            ref,
            modelRef,
            composerModel,
            testUtilities,
            formattingFunctions,
            onError,
            wysiwygProps?.inputEventProcessor,
        );

    useEditorFocus(ref, wysiwygProps?.isAutoFocusEnabled);

    const memoisedMappedSuggestion = useMemo(
        () => mapSuggestion(suggestion),
        [suggestion],
    );

    return {
        ref,
        isWysiwygReady: areListenersReady,
        wysiwyg: formattingFunctions,
        content,
        actionStates,
        debug: {
            modelRef,
            testRef,
            resetTestCase: testUtilities.onResetTestCase,
            traceAction: testUtilities.traceAction,
        },
        suggestion: memoisedMappedSuggestion,
        messageContent: composerModel?.get_content_as_message_html() ?? null,
    };
}
