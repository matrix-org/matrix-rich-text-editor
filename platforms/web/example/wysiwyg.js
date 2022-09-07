"use strict"

import init, { new_composer_model, new_composer_model_from_html } from './generated/wysiwyg.js';

let composer_model;
let editor;
let button_bold;
let button_italic;
let button_list_ordered;
let button_list_unordered;
let button_redo;
let button_strike_through;
let button_underline;
let button_undo;
let dom;
let testcase;
let reset_testcase;

let actions = [];
let last_update_html = "";

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

    button_list_ordered = document.getElementById('button_list_ordered');
    button_list_ordered.addEventListener('click', button_list_ordered_click);
    button_list_ordered.href = "";

    button_list_unordered = document.getElementById('button_list_unordered');
    button_list_unordered.addEventListener('click', button_list_unordered_click);
    button_list_unordered.href = "";

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

    reset_testcase = document.getElementById('reset_testcase');
    reset_testcase.addEventListener('click', resetTestcase);

    dom = document.getElementsByClassName('dom')[0];
    testcase = document.getElementsByClassName('testcase')[0];

    document.addEventListener('selectionchange', selectionchange);
    refresh_dom();
    editor.focus();
}

function resetTestcase() {
    let [start, end] = get_current_selection();
    actions = [
        ["replace_text", last_update_html],
        ["select", start, end],
    ];
    update_testcase(null);
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
            last_update_html = repl.replacement_html;
        }
        refresh_dom();
    }
}

function refresh_dom() {
    dom.innerHTML = "";
    const doc = composer_model.document();
    let idcounter = 0;

    function t(parent, name, text, attrs) {
        const tag = document.createElement(name);
        if (text) {
            tag.innerHTML = text;
        }
        if (attrs) {
            for (const [name, value] of attrs.entries()) {
                const attr = document.createAttribute(name);
                if (value !== null) {
                    attr.value = value;
                }
                tag.setAttributeNode(attr);
            }
        }
        parent.appendChild(tag);
        return tag;
    }

    function write_children(node, html) {
        const list = t(html, "ul");
        list.className = `group_${idcounter % 10}`;
        const children = node.children(composer_model);
        let child;
        while (child = children.next()) {
            if (child.node_type(composer_model) === "container") {
                let id = `dom_${idcounter}`;
                idcounter++;
                const li = t(list, "li");
                t(
                    li,
                    "input",
                    null,
                    new Map([
                        ["type", "checkbox"],
                        ["id", id],
                        ["checked", null]
                    ])
                );
                t(li, "label", child.tag(composer_model), new Map([["for", id]]));
                id++;
                write_children(child, li);
            } else {
                t(list, "li", `"${child.text(composer_model)}"`);
            }
        }
    }

    write_children(doc, dom);
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

function button_list_ordered_click(e) {
    send_input(e, "insertOrderedList");
}

function button_list_unordered_click(e) {
    send_input(e, "insertUnorderedList");
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

function get_current_selection() {
    const s = document.getSelection();
    // We should check that the selection is happening within the editor!
    // If anchor or focus are outside editor but not both, we should
    // change the selection, cutting off at the edge.
    // This should be done when we convert to React
    // Internal task for changing to React: PSU-721
    const start = codeunit_count(editor, s.anchorNode, s.anchorOffset);
    const end = codeunit_count(editor, s.focusNode, s.focusOffset);

    return [start, end];
}

function selection_according_to_actions(actions) {
    for (let i = actions.length - 1; i >= 0; i--) {
        const action = actions[i];
        console.log(action);
        if (action[0] === "select") {
            return [action[1], action[2]];
        }
    }
    return [-1, -1];
}

function selectionchange() {
    const [start, end] = get_current_selection();

    const prev_start = composer_model.selection_start();
    const prev_end = composer_model.selection_end();

    const [act_start, act_end] = selection_according_to_actions(actions);

    // Ignore selection changes that do nothing
    if (
        start === prev_start &&
        start === act_start &&
        end === prev_end &&
        end === act_end
    ) {
        return;
    }

    // Ignore selection changes that just reverse the selection - all
    // backwards selections actually do this, because the browser can't
    // support backwards selections.
    if (
        start === prev_end &&
        start === act_end &&
        end === prev_start &&
        end === act_start
    ) {
        return;
    }

    action(composer_model.select(start, end), "select", start, end);
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

            const endNodeBeforeStartNode = (
                start.node.compareDocumentPosition(end.node)
                    & Node.DOCUMENT_POSITION_PRECEDING
            );

            const sameNodeButEndOffsetBeforeStartOffset = (
                start.node === end.node && end.offset < start.offset
            );

            // Ranges must always have start before end
            if (
                endNodeBeforeStartNode
                || sameNodeButEndOffsetBeforeStartOffset
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

function action(update, nme, value1, value2) {
    if (value2 !== undefined) {
        console.debug(`composer_model.${nme}(${value1}, ${value2})`);
    } else if (value1 !== undefined) {
        console.debug(`composer_model.${nme}(${value1})`);
    } else {
        console.debug(`composer_model.${nme}()`);
    }

    actions.push([nme, value1, value2]);

    update_testcase(update);

    return update;
}

function add_selection(text, start, end) {
    const temp_model = new_composer_model_from_html(text);
    temp_model.select(start, end);
    return temp_model.to_example_format();
}

function update_testcase(update) {
    let html = editor.innerHTML;
    if (update) {
        // TODO: if (replacement_html !== html) SHOW AN ERROR?
        // TODO: handle other types of update (not just replace_all)
        html = update.text_update().replace_all?.replacement_html;
    }

    testcase.innerText = generate_testcase(
        actions, composer_model.to_example_format()
    );

    testcase.scrollTo(0, testcase.scrollTopMax);
}

function generate_testcase(actions, html) {
    let ret = "";

    function add(name, value1, value2) {
        if (name === "select") {
            ret += (
                "model.select("
                + `Location::from(${value1}), `
                + `Location::from(${value2}));\n`
            );
        } else if (value2 !== undefined) {
            ret += `model.${name}(${value1 ?? ""}, ${value2});\n`;
        } else if (name === "replace_text") {
            ret += `model.${name}("${value1 ?? ""}");\n`;
        } else {
            ret += `model.${name}(${value1 ?? ""});\n`;
        }
    }

    function start() {
        let text = add_selection(collected, selection[0], selection[1]);
        ret += `let mut model = cm("${text}");\n`;
    }

    let last_name = null;
    let collect_mode = true;
    let collected = "";
    let selection = [0, 0];
    for (const [name, value1, value2] of actions) {
        if (collect_mode) {
            if (name === "replace_text") {
                collected += value1;
            } else if (name === "select") {
                selection = [value1, value2];
            } else {
                collect_mode = false;
                start();
                add(name, value1, value2);
            }
        } else if (last_name === "select" && name === "select") {
            const nl = ret.lastIndexOf("\n", ret.length - 2);
            if (nl > -1) {
                ret = ret.substring(0, nl) + "\n";
                add(name, value1, value2);
            }
        } else {
            add(name, value1, value2);
        }
        last_name = name;
    }

    if (collect_mode) {
        start();
    }

    ret += `assert_eq!(tx(&model), "${html}");\n`;

    return ret;
}

function process_input(e) {
    switch (e.inputType) {
        case "deleteContentBackward":
            return action(composer_model.backspace(), "backspace");
        case "deleteContentForward":
            return action(composer_model.delete(), "delete");
        case "formatBold":
            return action(composer_model.bold(), "bold");
        case "formatItalic":
            return action(composer_model.italic(), "italic");
        case "formatStrikeThrough":
            return action(composer_model.strike_through(), "strike_through");
        case "formatUnderline":
            return action(composer_model.underline(), "underline");
        case "historyRedo":
            return action(composer_model.redo(), "redo");
        case "historyUndo":
            return action(composer_model.undo(), "undo");
        case "insertFromPaste":
        {
            const data = e.dataTransfer.getData("text");
            return action(
                composer_model.replace_text(data),
                "replace_text",
                data
            );
        }
        case "insertOrderedList":
            return action(
                composer_model.ordered_list(),
                "ordered_list"
            );
        case "insertParagraph":
            return action(composer_model.enter(), "enter");
        case "insertText":
            return action(
                composer_model.replace_text(e.data),
                "replace_text",
                e.data
            );
        case "insertUnorderedList":
            return action(
                composer_model.unordered_list(),
                "unordered_list"
            );
        default:
            // We should cover all of
            // https://rawgit.com/w3c/input-events/v1/index.html#interface-InputEvent-Attributes
            // Internal task to make sure we cover all inputs: PSU-740
            console.error(`Unknown input type: ${e.inputType}`);
            console.error(e);
            return null;
    }
}

export {
    wysiwyg_run,
    codeunit_count,
    node_and_offset,
    generate_testcase,
    selection_according_to_actions
};
