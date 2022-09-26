import { RefObject, useCallback, useMemo, useRef, useState } from 'react';

import { ComposerModel } from '../../generated/wysiwyg';
import { Actions } from './types';
import { getSelectionAccordingToActions, resetTestCase, traceAction } from './utils';

export function useTestCases(editorRef: RefObject<HTMLElement | null>, composerModel: ComposerModel | null) {
    const testRef = useRef<HTMLDivElement>(null);
    const [actions, setActions] = useState<Actions>([]);

    const [editorHtml, setEditorHtml] = useState<string>('');

    const memorizedTraceAction = useMemo(
        () => traceAction(testRef.current, actions, composerModel), [testRef, actions, composerModel],
    );

    const memorizedGetSelection = useMemo(() => getSelectionAccordingToActions(actions), [actions]);

    const onResetTestCase = useCallback(() => editorRef.current && testRef.current && composerModel &&
        setActions(resetTestCase(editorRef.current, testRef.current, composerModel, editorHtml)),
    [editorRef, testRef, composerModel, editorHtml],
    );

    return {
        testRef,
        utilities: {
            traceAction: memorizedTraceAction,
            getSelectionAccordingToActions: memorizedGetSelection,
            onResetTestCase,
            setEditorHtml,
        },
    };
}
