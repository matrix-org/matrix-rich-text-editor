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

import { ComposerModel, DomHandle } from '../generated/wysiwyg';

export function computeSelectionOffset(node: Node, offset?: number): number {
    if (node && node.nodeType === Node.TEXT_NODE) {
        return offset ?? node.textContent?.length ?? 0;
    } else if (node.hasChildNodes()) {
        return Array.from(node.childNodes)
            .map((childNode) => computeSelectionOffset(childNode))
            .reduce((prev, curr) => prev + curr, 0);
    } else {
        return 0;
    }
}

export function refreshComposerView(
    node: HTMLElement,
    composerModel: ComposerModel,
) {
    node.innerHTML = '';
    const doc = composerModel.document();
    let idCounter = 0;

    // TODO: use HTMLAttributes or similar to accept only valid HTML attributes
    function createNode(
        parent: Node,
        name: string,
        text?: string | null,
        attrs?: Map<string, string | null>,
    ) {
        const tag = document.createElement(name);
        if (text) {
            tag.innerText = text.replace('\u200b', '~');
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

    function writeChildren(node: DomHandle, html: HTMLElement) {
        const list = createNode(html, 'ul');
        list.className = `group_${idCounter % 10}`;
        const children = node.children(composerModel);
        let child: DomHandle | undefined;
        while ((child = children.next())) {
            const nodeType: string = child.node_type(composerModel);
            if (nodeType === 'container') {
                // TODO: I'm a bit sceptical about this id :p (Florian)
                const id = idCounter;
                const domId = `dom_${id}`;
                idCounter++;
                const li = createNode(list, 'li');
                createNode(
                    li,
                    'input',
                    null,
                    new Map([
                        ['type', 'checkbox'],
                        ['id', domId],
                        ['checked', null],
                    ]),
                );
                createNode(
                    li,
                    'label',
                    child.tag(composerModel),
                    new Map([['for', domId]]),
                );
                writeChildren(child, li);
            } else if (nodeType === 'line_break') {
                createNode(list, 'li', 'br');
            } else if (nodeType === 'text') {
                const li = createNode(list, 'li');
                createNode(li, 'span', '"', new Map([['class', 'quote']]));
                createNode(li, 'span', `${child.text(composerModel)}`);
                createNode(li, 'span', '"', new Map([['class', 'quote']]));
            } else {
                console.error(`Unknown node type: ${nodeType}`);
            }
        }
    }

    writeChildren(doc, node);
}

export function replaceEditor(
    editor: HTMLElement,
    htmlContent: string,
    startUtf16Codeunit: number,
    endUtf16Codeunit: number,
) {
    editor.innerHTML = htmlContent + '<br />';

    const sr = () => {
        const range = document.createRange();

        let start = computeNodeAndOffset(editor, startUtf16Codeunit);
        let end = computeNodeAndOffset(editor, endUtf16Codeunit);

        if (start.node && end.node) {
            const endNodeBeforeStartNode =
                start.node.compareDocumentPosition(end.node) &
                Node.DOCUMENT_POSITION_PRECEDING;

            const sameNodeButEndOffsetBeforeStartOffset =
                start.node === end.node && end.offset < start.offset;

            // Ranges must always have start before end
            if (
                endNodeBeforeStartNode ||
                sameNodeButEndOffsetBeforeStartOffset
            ) {
                [start, end] = [end, start];
            }

            range.setStart(start.node, start.offset);
            range.setEnd(end.node, end.offset);
        } else {
            // Nothing found in selection: select the end of editor
            range.selectNodeContents(editor);
            range.collapse();
        }

        const sel = document.getSelection();
        if (sel) {
            sel.removeAllRanges();
            sel.addRange(range);
        }
    };

    sr();
}

/**
 * Find the node that is codeunits into currentNode, by traversing
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
export function computeNodeAndOffset(currentNode: Node, codeunits: number) {
    const isEmptyList =
        currentNode.nodeName === 'LI' && !currentNode.hasChildNodes();
    if (currentNode.nodeType === Node.TEXT_NODE || isEmptyList) {
        if (codeunits <= (currentNode.textContent?.length || 0)) {
            return { node: currentNode, offset: codeunits };
        } else {
            return {
                node: null,
                offset: codeunits - (currentNode.textContent?.length || 0),
            };
        }
    } else if (currentNode.nodeName === 'BR') {
        // br tag acts like a text node of length 1, except if we're at
        // the end of it, we don't return it - instead we move on to
        // the next node, which will be returned with an offset of 0.
        // This is because we are not allowed to make a Range which points
        // to a br node with offset 1.
        if (codeunits === 0) {
            return { node: currentNode, offset: 0 };
        } else {
            return {
                node: null,
                offset: codeunits - 1,
            };
        }
    } else {
        for (const ch of currentNode.childNodes) {
            const ret = computeNodeAndOffset(ch, codeunits);
            if (ret.node) {
                return { node: ret.node, offset: ret.offset };
            } else {
                codeunits = ret.offset;
            }
        }
        return { node: null, offset: codeunits };
    }
}

/**
 * Find the start and end code units of the supplied selection in the supplied
 * editor.
 */
export function getCurrentSelection(
    editor: HTMLElement,
    selection: Selection | null,
) {
    if (!selection) {
        return [0, 0];
    }

    // We should check that the selection is happening within the editor!
    // If anchor or focus are outside editor but not both, we should
    // change the selection, cutting off at the edge.
    // This should be done when we convert to React
    // Internal task for changing to React: PSU-721
    const start =
        (selection.anchorNode &&
            countCodeunit(
                editor,
                selection.anchorNode,
                selection.anchorOffset,
            )) ||
        0;
    const end =
        (selection.focusNode &&
            countCodeunit(
                editor,
                selection.focusNode,
                selection.focusOffset,
            )) ||
        0;

    return [start, end];
}

/**
 * How many codeunits are there inside node, stopping counting if you get to
 * stopAtNode?
 */
function textLength(node: Node, stopAtNode: Node): number {
    if (node.nodeType === Node.TEXT_NODE) {
        return node.textContent?.length ?? 0;
    } else if (node.nodeName === 'BR') {
        // Treat br tags as being 1 character long
        return 1;
    } else {
        // Add up lengths until we hit the stop node.
        let sum = 0;
        for (const ch of node.childNodes) {
            if (ch === stopAtNode) {
                break;
            }
            sum += textLength(ch, stopAtNode);
        }
        return sum;
    }
}

/**
 * If nodeToFind is inside currentNode, return the number of codeunits you need
 * to count through currentNode to get to nodeToFind plus how many codeunits
 * through nodeToFind to get to offsetToFind.
 * Or, if nodeToFind is not a text node, count how many code units through
 * currentNode you need to count before you get to the offsetToFind-th child of
 * nodeToFind.
 *
 * Returns { found: true, offset: <the answer> } if nodeToFind is inside
 * CurrentNode and offsetToFind is within the length of nodeToFind,
 * or { found: false, 0 } if not.
 */
function findCharacter(
    currentNode: Node,
    nodeToFind: Node,
    offsetToFind: number,
): {
    found: boolean;
    offset: number;
} {
    if (currentNode === nodeToFind) {
        // We've found the right node
        if (currentNode.nodeType === Node.TEXT_NODE) {
            // Text node - use the offset to know where we are
            if (offsetToFind > (currentNode.textContent?.length ?? 0)) {
                // If the offset is wrong, we didn't find it
                return { found: false, offset: 0 };
            } else {
                // Otherwise, we did
                return { found: true, offset: offsetToFind };
            }
        } else {
            // Non-text node - offset is the index of the selected node
            // within currentNode.
            // Add up the sizes of all the nodes before offset.
            const ret = textLength(
                currentNode,
                currentNode.childNodes[offsetToFind],
            );
            return { found: true, offset: ret };
        }
    } else {
        // We have not found the right node yet

        if (currentNode.nodeType === Node.TEXT_NODE) {
            // Return how many steps forward we progress by skipping
            // this node.
            return {
                found: false,
                offset: currentNode.textContent?.length ?? 0,
            };
        } else if (currentNode.nodeName === 'BR') {
            // Treat br tags as being 1 character long
            return { found: false, offset: 1 };
        } else {
            // Add up all the amounts we need to progress by skipping
            // nodes inside this one.
            let sum = 0;
            for (const ch of currentNode.childNodes) {
                const cp = findCharacter(ch, nodeToFind, offsetToFind);
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

/**
 * Given a position in the document, count how many codeunits through the
 * editor that position is, by counting from the beginning of the editor,
 * traversing subnodes, until we hit the supplied position.
 *
 * If node is a text node, this means count codeunits until we hit the
 * offset-th codeunit of node.
 *
 * If node is not a text node, this cound codeunits until we hit the offset-th
 * child of node.
 *
 * A "codeunit" here means a UTF-16 code unit.
 *
 * Returns the number of codeunits you need to count through editor to get to
 * the supplied position, or -1 if node is not inside editor.
 */
export function countCodeunit(
    editor: HTMLElement,
    node: Node,
    offset: number,
): number {
    const ret = findCharacter(editor, node, offset);
    if (ret.found) {
        return ret.offset;
    } else {
        return -1;
    }
}
