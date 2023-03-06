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

import { RefObject, useCallback, useEffect, useMemo, useRef } from 'react';

import { InputEventProcessor } from './types.js';
import { useFormattingFunctions } from './useFormattingFunctions';
import { useComposerModel } from './useComposerModel';
import { useListeners } from './useListeners';
import { useTestCases } from './useTestCases';
import { SuggestionPattern } from '../generated/wysiwyg.js';

export { richToPlain, plainToRich } from './conversion';

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

    const { composerModel, initModel } = useComposerModel(
        ref,
        wysiwygProps?.initialContent,
    );
    const { testRef, utilities: testUtilities } = useTestCases(
        ref,
        composerModel,
    );

    const formattingFunctions = useFormattingFunctions(ref, composerModel);

    const onError = useCallback(
        (content?: string) => initModel(content),
        [initModel],
    );

    const { content, actionStates, areListenersReady, suggestion } =
        useListeners(
            ref,
            modelRef,
            composerModel,
            testUtilities,
            formattingFunctions,
            onError,
            wysiwygProps?.initialContent,
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
    };
}

function getSuggestionChar(suggestion: SuggestionPattern) {
    const suggestionMap = ['@', '#', '/'];
    return suggestionMap[suggestion.key];
}

function getSuggestionType(suggestion: SuggestionPattern) {
    switch (suggestion.key) {
        case 0:
        case 1:
            return 'mention';
        case 2:
            return 'command';
        default:
            return 'unknown';
    }
}

// TODO use this when passing the output out from the hook => this way we can
// keep the state consistent, but use 'MappedSuggestion' type on the React side
function mapSuggestion(suggestion: SuggestionPattern | null) {
    if (suggestion === null) return suggestion;
    return {
        ...suggestion,
        keyChar: getSuggestionChar(suggestion),
        type: getSuggestionType(suggestion),
    };
}
