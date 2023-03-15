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

import { MouseEvent as ReactMouseEvent } from 'react';

import {
    ComposerModel,
    MenuStateUpdate,
    SuggestionPattern,
} from '../../generated/wysiwyg';
import { processEvent, processInput } from '../composer';
import {
    getCurrentSelection,
    refreshComposerView,
    replaceEditor,
} from '../dom';
import {
    BlockType,
    FormattingFunctions,
    InputEventProcessor,
    WysiwygInputEvent,
} from '../types';
import { TestUtilities } from '../useTestCases/types';
import { AllActionStates } from '../types';
import { mapToAllActionStates } from './utils';
import { LinkEvent } from './types';

/**
 * Send a custom event named wysiwygInput
 * See {FormatBlockEvent} for the event structure
 * @param {HTMLElement} editor
 * @param {BlockType} blockType
 * @param {ReactMouseEvent<HTMLElement, MouseEvent> | KeyboardEvent} e
 * @param {String} data
 */
export function sendWysiwygInputEvent(
    editor: HTMLElement,
    blockType: BlockType,
    e?: ReactMouseEvent<HTMLElement, MouseEvent> | KeyboardEvent,
    data?: string | LinkEvent['data'],
) {
    e?.preventDefault();
    e?.stopPropagation();
    editor.dispatchEvent(
        new CustomEvent('wysiwygInput', { detail: { blockType, data } }),
    );
}

/**
 * Return the blockType associated to a shortcut
 * @param {KeyboardEvent} e
 * @returns {BlockType | null}
 */
function getInputFromKeyDown(
    e: KeyboardEvent,
    composerModel: ComposerModel,
    formattingFunctions: FormattingFunctions,
    editor: HTMLElement,
    inputEventProcessor?: InputEventProcessor,
): BlockType | null {
    if (e.shiftKey && e.altKey) {
        switch (e.key) {
            case '5':
                return 'formatStrikeThrough';
        }
    }

    if (e.ctrlKey || e.metaKey) {
        switch (e.key) {
            case 'b':
                return 'formatBold';
            case 'i':
                return 'formatItalic';
            case 'u':
                return 'formatUnderline';
            case 'e':
                return 'formatInlineCode';
            case 'y':
                return 'historyRedo';
            case 'z':
                return 'historyUndo';
            case 'Z':
                return 'historyRedo';
            case 'Enter':
                return 'sendMessage';
            case 'Backspace':
                return 'deleteSoftLineBackward';
        }
    }

    // we have to handle pill deletion before the `deleteContentBackward` event fires
    const s = document.getSelection();
    if (e.key === 'Backspace' && s && s.isCollapsed) {
        // CASE 1 - going from the nbsp following a text node into a mention
        // if (
        //     s.anchorNode?.textContent === '\u00A0' &&
        //     s.anchorNode?.previousSibling?.firstChild?.parentElement?.getAttribute(
        //         'contentEditable',
        //     ) === 'false'
        // ) {
        //     console.log('entering the pill from single nbsp after the pill');
        //     // expand the selection, then delete
        //     s.setBaseAndExtent(
        //         s.anchorNode,
        //         s.anchorOffset,
        //         s.anchorNode.previousSibling.firstChild,
        //         0,
        //     );
        //     document.dispatchEvent(new CustomEvent('selectionchange'));
        //     return 'deleteContentBackward';
        // }
        // CASE 2 - going from beginning of text node into mention
        if (
            (s.anchorNode?.nodeName === '#text' ||
                s.anchorNode?.nodeName === 'BR') &&
            s.anchorOffset === 0 &&
            s.anchorNode?.previousSibling?.firstChild?.parentElement?.getAttribute(
                'contentEditable',
            ) === 'false'
        ) {
            console.log('entering the pill from text node after the pill');
            // expand the selection, thefn delete
            s.setBaseAndExtent(
                s.anchorNode,
                s.anchorOffset,
                s.anchorNode.previousSibling.firstChild,
                0,
            );
            document.dispatchEvent(new CustomEvent('selectionchange'));
            return 'deleteContentBackward';
        }

        // CASE 3 - pressing backspace at the beginning of the composer, just
        // ignore it (it can cause issues with pills)
        // nb 1 to include new hr tag
        if (s.anchorNode === editor && s.anchorOffset === 1) {
            console.log('trying to backspace at beginning of composer');
            // put the cursor back at the beginning
            s.setBaseAndExtent(editor, 1, editor, 1);
            return null; // do no deletion
        }
    }
    if (e.key === 'Delete' && s && s.isCollapsed) {
        // CASE 1 - going from text node into the mention AT START OF THE EDITOR
        if (
            s.anchorNode === editor &&
            s.anchorOffset === 1 && // nb changed to 1 due due presence of caretLine
            editor.children[1] &&
            editor.children[1].getAttribute('contentEditable') === 'false'
        ) {
            console.log('entering the pill from start of editor');
            // expand the selection, then delete
            // we know here it's an a tag with a single text node inside it
            // nb don't clip the nbsp off here, for caret purposes
            s.setBaseAndExtent(
                editor.children[1],
                0,
                editor.children[1].nextSibling,
                0,
            );
            document.dispatchEvent(new CustomEvent('selectionchange'));
            return 'deleteContentForward';
        }
        // CASE 2 - just going from end of a regular text node into the mention
        if (
            s.anchorNode &&
            s.anchorOffset === s.anchorNode.textContent.length &&
            s.anchorNode.nodeName === '#text' &&
            s.anchorNode.nextSibling?.firstChild?.parentElement?.getAttribute(
                'contentEditable',
            ) === 'false'
        ) {
            console.log('entering the pill from a prior text node');
            // expand the selection, then delete
            // we know here it's an a tag with a single text node inside it,
            // and we want to remove the trailing nbsp as well
            s.setBaseAndExtent(
                s.anchorNode.nextSibling,
                0,
                s.anchorNode.nextSibling.nextSibling,
                1,
            );
            document.dispatchEvent(new CustomEvent('selectionchange'));
            return 'deleteContentForward';
        }
    }

    processEvent(
        e,
        {
            actions: formattingFunctions,
            content: () => composerModel.get_content_as_html(),
        },
        editor,
        inputEventProcessor,
    );
    return null;
}

/**
 * Event listener for keydown event
 * @param {KeyboardEvent} e
 * @param {HTMLElement} editor
 */
export function handleKeyDown(
    e: KeyboardEvent,
    editor: HTMLElement,
    composerModel: ComposerModel,
    formattingFunctions: FormattingFunctions,
    inputEventProcessor?: InputEventProcessor,
) {
    const inputType = getInputFromKeyDown(
        e,
        composerModel,
        formattingFunctions,
        editor,
        inputEventProcessor,
    );
    if (inputType) {
        sendWysiwygInputEvent(editor, inputType, e);
    }
}

/**
 * Extract the action states from the menu state of the composer
 * @param {MenuStateUpdate} menuStateUpdate menu state update from the composer
 * @returns {AllActionStates}
 */
export function extractActionStates(
    menuStateUpdate: MenuStateUpdate,
): AllActionStates {
    return mapToAllActionStates(menuStateUpdate.action_states);
}

/**
 * Event listener for WysiwygInputEvent
 * @param {WysiwygInputEvent} e
 * @param {HTMLElement} editor
 * @param {ComposerModel} composerModel
 * @param {HTMLElement | null} modelNode
 * @param {TestUtilities} testUtilities
 * @param {FormattingFunctions} formattingFunctions
 * @param {InputEventProcessor} inputEventProcessor
 * @returns
 */
export function handleInput(
    e: WysiwygInputEvent,
    editor: HTMLElement,
    composerModel: ComposerModel,
    modelNode: HTMLElement | null,
    testUtilities: TestUtilities,
    formattingFunctions: FormattingFunctions,
    suggestion: SuggestionPattern | null,
    inputEventProcessor?: InputEventProcessor,
):
    | {
          content?: string;
          actionStates: AllActionStates | null;
          suggestion: SuggestionPattern | null;
      }
    | undefined {
    const update = processInput(
        e,
        composerModel,
        testUtilities.traceAction,
        formattingFunctions,
        editor,
        suggestion,
        inputEventProcessor,
    );
    if (update) {
        const repl = update.text_update().replace_all;
        if (repl) {
            replaceEditor(
                editor,
                repl.replacement_html,
                repl.start_utf16_codeunit,
                repl.end_utf16_codeunit,
            );
            testUtilities.setEditorHtml(repl.replacement_html);
        }
        editor.focus();

        // Only when
        if (modelNode) {
            refreshComposerView(modelNode, composerModel);
        }

        const menuStateUpdate = update.menu_state().update();
        const menuActionUpdate = update
            .menu_action()
            .suggestion()?.suggestion_pattern;

        const actionStates = menuStateUpdate
            ? extractActionStates(menuStateUpdate)
            : null;

        const suggestion = menuActionUpdate || null;

        const res = {
            content: repl?.replacement_html,
            actionStates,
            suggestion,
        };

        return res;
    }
}

/**
 * Event listener for selectionChange event
 * @param {Editor} editor
 * @param {ComposerModel} composeModel
 * @param {TestUtilities}
 * @returns
 */
export function handleSelectionChange(
    editor: HTMLElement,
    composeModel: ComposerModel,
    { traceAction, getSelectionAccordingToActions }: TestUtilities,
): AllActionStates | undefined {
    const [start, end] = getCurrentSelection(editor, document.getSelection());
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
    const update = composeModel.select(start, end);
    traceAction(null, 'select', start, end);

    const menuStateUpdate = update.menu_state().update();

    if (menuStateUpdate) {
        return extractActionStates(menuStateUpdate);
    }
}
