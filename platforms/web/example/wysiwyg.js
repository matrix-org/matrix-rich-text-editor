import init, { new_composer_model } from './generated/wysiwyg.js';

let composer_model;
let editor;
let button_bold;

async function wysiwyg_run() {
    await init();

    console.debug(`new_composer_model`);
    composer_model = new_composer_model();

    editor = document.getElementsByClassName('editor')[0];
    editor.addEventListener('input', editor_input);
    editor.addEventListener("keydown", editor_keydown);

    button_bold = document.getElementById('button_bold');
    button_bold.addEventListener('click', button_bold_click);
    button_bold.href = "";

    button_italic = document.getElementById('button_italic');
    button_italic.addEventListener('click', button_italic_click);
    button_italic.href = "";

    button_redo = document.getElementById('button_redo');
    button_redo.addEventListener('click', button_redo_click);
    button_redo.href = "";

    button_strike_through = document.getElementById('button_strike_through');
    button_strike_through.addEventListener('click', button_strike_through_click);
    button_strike_through.href = "";

    button_underline = document.getElementById('button_underline');
    button_underline.addEventListener('click', button_underline_click);
    button_underline.href = "";

    button_undo = document.getElementById('button_undo');
    button_undo.addEventListener('click', button_undo_click);
    button_undo.href = "";

    document.addEventListener('selectionchange', selectionchange);
    editor.focus();
}

function editor_input(e) {
    const update = process_input(e);
    if (update) {
        const repl = update.text_update().replace_all;
        if (repl) {
            replace_editor(
                repl.replacement_html,
                repl.start_utf16_codeunit,
                repl.end_utf16_codeunit
            );
        }
    }
}

function send_input(e, inputType) {
    editor.dispatchEvent(new InputEvent('input', { inputType }));
    e.preventDefault();
}

function input_for_editor_keydown(e) {
    if (e.shiftKey && e.altKey) {
        switch (e.key) {
            case '5': return "formatStrikeThrough";
        }
    }

    if ((e.ctrlKey || e.metaKey)) {
        switch (e.key) {
            case 'b': return "formatBold";
            case 'i': return "formatItalic";
            case 'u': return "formatUnderline";
            case 'y': return "historyRedo";
            case 'z': return "historyUndo";
            case 'Z': return "historyRedo";
        }
    }

    return null;
}

function editor_keydown(e) {
    let inputType = input_for_editor_keydown(e);
    if (inputType) {
        send_input(e, inputType);
    }
}

function button_bold_click(e) {
    send_input(e, "formatBold");
}

function button_italic_click(e) {
    send_input(e, "formatItalic");
}

function button_strike_through_click(e) {
    send_input(e, "formatStrikeThrough");
}

function button_redo_click(e) {
    send_input(e, "historyRedo");
}

function button_underline_click(e) {
    send_input(e, "formatUnderline");
}

function button_undo_click(e) {
    send_input(e, "historyUndo");
}

function selectionchange() {
    const s = document.getSelection();
    // We should check that the selection is happening within the editor!
    // If anchor or focus are outside editor but not both, we should
    // change the selection, cutting off at the edge.
    // This should be done when we convert to React
    // Internal task for changing to React: PSU-721
    const start = codeunit_count(editor, s.anchorNode, s.anchorOffset);
    const end = codeunit_count(editor, s.focusNode, s.focusOffset);

    console.debug(`composer_model.select(${start}, ${end})`);
    composer_model.select(start, end);
}

/**
 * Given a position in the document, count how many codeunits through the
 * editor that position is, by counting from the beginning of the editor,
 * traversing subnodes, until we hit the supplied position.
 *
 * "Position" means a node and an offset, meaning the offset-th codeunit in
 * node.
 *
 * A "codeunit" here means a UTF-16 code unit.
 */
function codeunit_count(editor, node, offset) {
    function impl(current_node, offset) {
        if (current_node === node) {
            // We've found the right node
            if (
                current_node.nodeType === Node.TEXT_NODE
                && offset > current_node.textContent.length
            ) {
                // If the offset is wrong, we didn't find it
                return { found: false, offset: 0 };
            } else {
                // Otherwise, we did
                return { found: true, offset };
            }
        } else {
            // We have not found the right node yet
            if (current_node.nodeType === Node.TEXT_NODE) {
                // Return how many steps forward we progress by skipping
                // this node.
                return {
                    found: false,
                    offset: current_node.textContent.length
                };
            } else {
                // Add up all the amounts we need progress by skipping
                // nodes inside this one.
                let sum = 0;
                for (const ch of current_node.childNodes) {
                    const cp = impl(ch, offset);
                    if (cp.found) {
                        // We found it! Return how far we walked to find it
                        return { found: true, offset: sum + cp.offset };
                    } else {
                        // We didn't find it - remember how much to skip
                        sum += cp.offset;
                    }
                }
                return { found: false, offset: sum };
            }
        }
    }

    const ret = impl(editor, offset);
    if (ret.found) {
        return ret.offset;
    } else {
        return -1;
    }
}

/**
 * Find the node that is codeunits into current_node, by traversing
 * its subnodes.
 *
 * Returns {
 *   node: // The node that contains the codeunits-th codeunit
 *   offset: // How far into the found node we can find that codeunit
 * }
 *
 * If there are not that many codeunits, returns { node: null, offset: n }
 * where n is the number of codeunits past the end of our last subnode we would
 * need to go to find the requested position.
 *
 * A "codeunit" here means a UTF-16 code unit.
 */
function node_and_offset(current_node, codeunits) {
    if (current_node.nodeType === Node.TEXT_NODE) {
        if (codeunits <= current_node.textContent.length) {
            return { node: current_node, offset: codeunits };
        } else {
            return {
                node: null,
                offset: codeunits - current_node.textContent.length
            };
        }
    } else {
        for (const ch of current_node.childNodes) {
            const ret = node_and_offset(ch, codeunits);
            if (ret.node) {
                return { node: ret.node, offset: ret.offset };
            } else {
                codeunits = ret.offset;
            }
        }
        return { node: null, offset: codeunits };
    }
}

function replace_editor(html, start_utf16_codeunit, end_utf16_codeunit) {
    console.log(
        "replace_editor",
        html,
        start_utf16_codeunit,
        end_utf16_codeunit
    );
    editor.innerHTML = html;

    const sr = () => {

        const range = document.createRange();

        let start = node_and_offset(editor, start_utf16_codeunit);
        let end = node_and_offset(editor, end_utf16_codeunit);

        if (start.node && end.node) {
            // Ranges must always have start before end
            if (
                start.node.compareDocumentPosition(end.node)
                    & Node.DOCUMENT_POSITION_PRECEDING
            ) {
                [start, end] = [end, start];
            }

            range.setStart(start.node, start.offset);
            range.setEnd(end.node, end.offset);
            var sel = document.getSelection();
            sel.removeAllRanges();
            sel.addRange(range);
        } else {
            console.error("Failed to find offsets", start, end);
        }
    };

    sr();
}

function process_input(e) {
    switch (e.inputType) {
        case "deleteContentBackward":
            console.debug(`composer_model.backspace()`);
            return composer_model.backspace();
        case "deleteContentForward":
            console.debug(`composer_model.delete()`);
            return composer_model.delete();
        case "formatBold":
            console.debug(`composer_model.bold()`);
            return composer_model.bold();
        case "formatItalic":
            console.debug(`composer_model.italic()`);
            return composer_model.italic();
        case "formatStrikeThrough":
            console.debug(`composer_model.strike_through()`);
            return composer_model.strike_through();
        case "formatUnderline":
            console.debug(`composer_model.underline()`);
            return composer_model.underline();
        case "historyRedo":
            console.debug(`composer_model.redo()`);
            return composer_model.redo();
        case "historyUndo":
            console.debug(`composer_model.undo()`);
            return composer_model.undo();
        case "insertFromPaste":
        {
            const data = e.dataTransfer.getData("text");
            console.debug(`composer_model.replace_text(${data})`);
            return composer_model.replace_text(data);
        }
        case "insertParagraph":
            console.debug(`composer_model.enter()`);
            return composer_model.enter();
        case "insertText":
            console.debug(`composer_model.replace_text(${e.data})`);
            return composer_model.replace_text(e.data);
        default:
            // TODO: cover all of https://rawgit.com/w3c/input-events/v1/index.html#interface-InputEvent-Attributes
            console.error(`Unknown input type: ${e.inputType}`);
            console.error(e);
            return null;
    }
}

export { wysiwyg_run, codeunit_count, node_and_offset };
