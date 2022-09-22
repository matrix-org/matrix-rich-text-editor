import { MouseEvent as ReactMouseEvent } from 'react';

import { ComposerModel } from '../../../generated/wysiwyg';
import { processInput } from '../composer';
import { getCurrentSelection, refreshComposerView, replaceEditor } from '../dom';
import { WysiwygInputEvent } from '../types';
import { TestUtilities } from '../useTestCases/types';

export function sendInputEvent(
    e: ReactMouseEvent<HTMLElement, MouseEvent> | KeyboardEvent,
    editor: HTMLElement,
    inputType: InputEvent['inputType'],
) {
    e.preventDefault();
    e.stopPropagation();
    editor.dispatchEvent(new InputEvent('input', { inputType }));
}

function getInputFromKeyDown(e: KeyboardEvent) {
    if (e.shiftKey && e.altKey) {
        switch (e.key) {
            case '5': return 'formatStrikeThrough';
        }
    }

    if ((e.ctrlKey || e.metaKey)) {
        switch (e.key) {
            case 'b': return 'formatBold';
            case 'i': return 'formatItalic';
            case 'u': return 'formatUnderline';
            case 'y': return 'historyRedo';
            case 'z': return 'historyUndo';
            case 'Z': return 'historyRedo';
        }
    }

    return null;
}

export function handleKeyDown(e: KeyboardEvent, editor: HTMLElement) {
    const inputType = getInputFromKeyDown(e);
    if (inputType) {
        sendInputEvent(e, editor, inputType);
    }
}

export function handleInput(
    e: WysiwygInputEvent,
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

export function handleSelectionChange(
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
    traceAction(null, 'select', start, end);
}

