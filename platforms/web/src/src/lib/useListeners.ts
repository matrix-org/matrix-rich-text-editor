import { RefObject, useEffect } from "react";

import { ComposerModel } from "../../generated/wysiwyg";
import { isInputEvent } from "./assert";
import { processInput } from "./composer";
import { getCurrentSelection, refreshComposerView, replaceEditor } from "./dom";
import { TestUtilities } from "./useTestCases";

function handleInput(
    e: InputEvent,
    editor: HTMLElement,
    composerModel: ComposerModel,
    modelNode: HTMLElement | null,
    testUtilities: TestUtilities,
) {
    const update = processInput(e, composerModel, testUtilities.traceAction);
    if (update) {
        const repl = update.text_update().replace_all;
        if (repl) {
            replaceEditor(editor,
                repl.replacement_html,
                repl.start_utf16_codeunit,
                repl.end_utf16_codeunit,
            );
            // todo test case
            testUtilities.setEditorHtml(repl.replacement_html);
            // last_update_html = repl.replacement_html;
        }

        // Only when
        if (modelNode) {
            refreshComposerView(modelNode, composerModel);
        }
    }
}

function handleSelectionChange(
    editor: HTMLElement,
    composeModel: ComposerModel,
    { traceAction, getSelectionAccordingToActions }: TestUtilities,
) {
    const isInEditor = document.activeElement === editor;

    // Skip the selection behavior when the focus is not in the editor
    if (!isInEditor) {
        return;
    }

    const [start, end] = getCurrentSelection(editor);

    const prevStart = composeModel.selection_start();
    const prevEnd = composeModel.selection_end();

    const [actStart, actEnd] = getSelectionAccordingToActions();

    // Ignore selection changes that do nothing
    if (
        start === prevStart &&
        start === actStart &&
        end === prevEnd &&
        end === actEnd
    ) {
        return;
    }

    // Ignore selection changes that just reverse the selection - all
    // backwards selections actually do this, because the browser can't
    // support backwards selections.
    if (
        start === prevEnd &&
        start === actEnd &&
        end === prevStart &&
        end === actStart
    ) {
        return;
    }
    composeModel.select(start, end);
    traceAction(null, "select", start, end);
}

export function useListeners(
    editorRef: RefObject<HTMLElement | null>,
    modelRef: RefObject<HTMLElement | null>,
    composerModel: ComposerModel | null,
    testUtilities: TestUtilities,
) {
    useEffect(() => {
        const editorNode = editorRef.current;
        if (!composerModel || !editorNode) {
            return;
        }

        // React uses SyntheticEvent (https://reactjs.org/docs/events.html) and doesn't catch manually fired event (myNode.dispatchEvent)
        const onInput = (e: Event) =>
            isInputEvent(e) && handleInput(
                e,
                editorNode,
                composerModel,
                modelRef.current,
                testUtilities,
            );
        editorNode.addEventListener('input', onInput);

        const onSelectionChange = () => handleSelectionChange(editorNode, composerModel, testUtilities);
        document.addEventListener('selectionchange', onSelectionChange);

        return () => {
            editorNode.removeEventListener('input', onInput);
            document.removeEventListener('selectionchange', onSelectionChange);
        };
    }, [editorRef, composerModel, modelRef, testUtilities]);
}
