import { ComposerModel } from '../../generated/wysiwyg';
import { WysiwygInputEvent } from './types';
import { TestUtilities } from './useTestCases/types';

export function processInput(
    e: WysiwygInputEvent,
    composerModel: ComposerModel,
    action: TestUtilities['traceAction'],
) {
    switch (e.inputType) {
        case 'deleteContentBackward':
            return action(composerModel.backspace(), 'backspace');
        case 'deleteContentForward':
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
        case 'insertFromPaste': {
            if (e.dataTransfer) {
                const data = e.dataTransfer.getData('text');
                return action(composerModel.replace_text(data), 'replace_text', data);
            }
            break;
        }
        case 'insertOrderedList':
            return action(composerModel.ordered_list(), 'ordered_list');
        case 'insertParagraph':
            return action(composerModel.enter(), 'enter');
        case 'insertText':
            if (e.data) {
                return action(
                    composerModel.replace_text(e.data),
                    'replace_text',
                    e.data,
                );
            }
            break;
        case 'insertUnorderedList':
            return action(composerModel.unordered_list(), 'unordered_list');
        default:
            // We should cover all of
            // https://rawgit.com/w3c/input-events/v1/index.html#interface-InputEvent-Attributes
            // Internal task to make sure we cover all inputs: PSU-740
            console.error(`Unknown input type: ${e.inputType}`);
            console.error(e);
            return null;
    }
}
