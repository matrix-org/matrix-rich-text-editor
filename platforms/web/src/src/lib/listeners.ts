import { ComposerModel } from "../../generated/wysiwyg";
import { processInput } from "./composer";
import { getCurrentSelection, refreshComposerView, replaceEditor } from "./dom";
import { action, getSelectionAccordingToActions } from "./testCase";

export function handleInput(
    e: InputEvent, editor:
    HTMLElement,
    composerModel: ComposerModel,
    modelNode: HTMLElement | null,
) {
    const update = processInput(e, composerModel);
    if (update) {
        const repl = update.text_update().replace_all;
        if (repl) {
            replaceEditor(editor,
                repl.replacement_html,
                repl.start_utf16_codeunit,
                repl.end_utf16_codeunit,
            );
            // todo test case
            // last_update_html = repl.replacement_html;
        }

        // Only when
        if (modelNode) {
            refreshComposerView(modelNode, composerModel);
        }
    }
}

export function handleSelectionChange(editor: HTMLElement, composeModel: ComposerModel) {
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
    action(null, "select", start, end);
}
