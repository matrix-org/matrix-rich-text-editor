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

import { ComposerModel } from '../generated/wysiwyg';
import {
    WysiwygInputEvent,
    InputEventProcessor,
    Wysiwyg,
    FormattingFunctions,
} from './types';
import { TestUtilities } from './useTestCases/types';

function processEvent(
    e: WysiwygInputEvent,
    wysiwyg: Wysiwyg,
    inputEventProcessor?: InputEventProcessor,
): WysiwygInputEvent | null {
    if (inputEventProcessor) {
        return inputEventProcessor(e, wysiwyg);
    } else {
        return e;
    }
}

export function processInput(
    e: WysiwygInputEvent,
    composerModel: ComposerModel,
    action: TestUtilities['traceAction'],
    formattingFunctions: FormattingFunctions,
    inputEventProcessor?: InputEventProcessor,
) {
    const event = processEvent(
        e,
        {
            actions: formattingFunctions,
            content: () => composerModel.get_html(),
        },
        inputEventProcessor,
    );
    if (!event) {
        return;
    }

    if (event instanceof ClipboardEvent) {
        const data = event.clipboardData?.getData('text/plain') ?? '';
        return action(composerModel.replace_text(data), 'paste');
    }

    switch (event.inputType) {
        case 'deleteContentBackward':
            return action(composerModel.backspace(), 'backspace');
        case 'deleteContentForward':
            return action(composerModel.delete(), 'delete');
        case 'deleteByCut':
            return action(composerModel.delete(), 'delete');
        case 'formatBold':
            return action(composerModel.bold(), 'bold');
        case 'formatItalic':
            return action(composerModel.italic(), 'italic');
        case 'formatStrikeThrough':
            return action(composerModel.strike_through(), 'strike_through');
        case 'formatUnderline':
            return action(composerModel.underline(), 'underline');
        case 'formatInlineCode':
            return action(composerModel.inline_code(), 'inline_code');
        case 'historyRedo':
            return action(composerModel.redo(), 'redo');
        case 'historyUndo':
            return action(composerModel.undo(), 'undo');
        case 'insertFromPaste':
            // Paste is already handled by catching the 'paste' event, which
            // results in a ClipboardEvent, handled above. Ideally, we would
            // do it here, but Chrome does not provide data inside this
            // InputEvent, only in the original ClipboardEvent.
            return;
        case 'insertOrderedList':
            return action(composerModel.ordered_list(), 'ordered_list');
        case 'insertLineBreak':
        case 'insertParagraph':
            return action(composerModel.enter(), 'enter');
        case 'insertText':
            if (event.data) {
                return action(
                    composerModel.replace_text(event.data),
                    'replace_text',
                    event.data,
                );
            }
            break;
        case 'insertUnorderedList':
            return action(composerModel.unordered_list(), 'unordered_list');
        case 'clear':
            return action(composerModel.clear(), 'clear');
        default:
            // We should cover all of
            // eslint-disable-next-line max-len
            // https://rawgit.com/w3c/input-events/v1/index.html#interface-InputEvent-Attributes
            // Internal task to make sure we cover all inputs: PSU-740
            console.error(`Unknown input type: ${event.inputType}`);
            console.error(e);
            return null;
    }
}
