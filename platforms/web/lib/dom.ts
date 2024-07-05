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

export function refreshComposerView(
    node: HTMLElement,
    composerModel: ComposerModel,
): void {
    node.innerHTML = '';
    const doc = composerModel.document();
    let idCounter = 0;

    // TODO: use HTMLAttributes or similar to accept only valid HTML attributes
    function createNode(
        parent: Node,
        name: string,
        text?: string | null,
        attrs?: Map<string, string | null>,
    ): HTMLElement {
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

    function writeChildren(node: DomHandle, html: HTMLElement): void {
        const list = createNode(html, 'ul');
        list.className = `group_${idCounter % 10}`;
        const children = node.children(composerModel);
        let child: DomHandle | undefined;
        while ((child = children.next_child())) {
            const nodeType: string = child.node_type(composerModel);
            if (nodeType === 'container') {
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
            } else if (nodeType === 'mention') {
                const li = createNode(list, 'li');
                createNode(
                    li,
                    'span',
                    '"Mention - ',
                    new Map([['class', 'quote']]),
                );
                createNode(li, 'span', `${child.text(composerModel)}`);
                createNode(li, 'span', '"', new Map([['class', 'quote']]));
            } else {
                console.error(`Unknown node type: ${nodeType}`);
            }
        }
    }

    writeChildren(doc, node);
}

export function selectContent(
    editor: HTMLElement,
    startUtf16Codeunit: number,
    endUtf16Codeunit: number,
): void {
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
        if (endNodeBeforeStartNode || sameNodeButEndOffsetBeforeStartOffset) {
            [start, end] = [end, start];
            if (!start.node || !end.node) throw new Error();
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
}

export function replaceEditor(
    editor: HTMLElement,
    htmlContent: string,
    startUtf16Codeunit: number,
    endUtf16Codeunit: number,
): void {
    editor.innerHTML = htmlContent + '<br />';
    selectContent(editor, startUtf16Codeunit, endUtf16Codeunit);
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
 *
 * When first called, currentNode is the editor and rootNode is undefined. On
 * recursive calls from inside this function, rootNode will be defined and will
 * be the editor node (which allows us to always select the editor if required
 * regardless of how deep we have gone into the tree)
 */
export function computeNodeAndOffset(
    currentNode: Node,
    codeunits: number,
    rootNode?: Node,
): {
    node: Node | null;
    offset: number;
} {
    const isEmptyListItem =
        currentNode.nodeName === 'LI' && !currentNode.hasChildNodes();

    const isTextNode = currentNode.nodeType === Node.TEXT_NODE;
    const isTextNodeInsideMention =
        isTextNode &&
        currentNode.parentElement?.hasAttribute('data-mention-type');

    if (isTextNodeInsideMention) {
        // Special casing for mention nodes. They will be a node with a single
        // text node child. We can therefore guarantee that the text node will
        // have both parent and grandparent nodes.

        // We _may_ need an extra offset if we're inside a p tag
        const shouldAddOffset = textNodeNeedsExtraOffset(currentNode);
        const extraOffset = shouldAddOffset ? 1 : 0;

        const remainingCodeunits = codeunits - extraOffset;

        // We have only _found_ the node if we have 0 or 1 remainingCodeunits
        // as we treat a mention as having length 1
        if (remainingCodeunits <= 1) {
            if (remainingCodeunits === 0) {
                // if we have hit the beginning of the node, we either want to
                // put the cursor at the end of the previous sibling (if it has
                // one) or at the 0th index of the parent otherwise
                if (currentNode.previousSibling) {
                    return {
                        node: currentNode.previousSibling,
                        offset: textLength(
                            currentNode.previousSibling,
                            Infinity,
                        ),
                    };
                } else {
                    return {
                        node: currentNode.parentNode?.parentNode ?? null,
                        offset: 0,
                    };
                }
            } else {
                // setting node to null means if we end up inside or at end of a
                // non-editable node somehow, we will return "node not found"
                // and so we will keep searching
                return { node: null, offset: 0 };
            }
        } else {
            return { node: null, offset: codeunits - extraOffset - 1 };
        }
    } else if (isTextNode) {
        // For a text node, we need to check to see if it needs an extra offset
        // which involves climbing the tree through it's ancestors checking for
        // any of the nodes that require the extra offset.
        const shouldAddOffset = textNodeNeedsExtraOffset(currentNode);
        const extraOffset = shouldAddOffset ? 1 : 0;

        // We also have a special case for a text node that is a single &nbsp;
        // which is used as a placeholder for an empty paragraph - we don't want
        // to count it's length
        if (isPlaceholderParagraphNode(currentNode)) {
            if (codeunits === 0) {
                // this is the only time we would 'find' this node
                return { node: currentNode, offset: codeunits };
            } else {
                // otherwise we need to keep looking, but count this as 0 length
                return { node: null, offset: codeunits - extraOffset };
            }
        }

        if (codeunits <= (currentNode.textContent?.length || 0)) {
            // we don't need to use that extra offset if we've found the answer
            return { node: currentNode, offset: codeunits };
        } else {
            // but if we haven't found that answer, apply the extra offset
            return {
                node: null,
                offset:
                    codeunits -
                    (currentNode.textContent?.length || 0) -
                    extraOffset,
            };
        }
    } else if (isEmptyListItem) {
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
        // We hit this case if we split a formatting node, eg
        // <u>something</u> => press enter => <p><u>something</u><p><u>|</u></p>
        if (isEmptyInlineNode(currentNode) && codeunits === 0) {
            return { node: currentNode, offset: codeunits };
        }

        for (const ch of currentNode.childNodes) {
            const ret = computeNodeAndOffset(
                ch,
                codeunits,
                rootNode || currentNode,
            );
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
): number[] {
    // return [0,0] when selection is null, or we have an empty editor
    const editorIsEmpty =
        editor.childNodes.length === 1 && editor.firstChild?.nodeName === 'BR';

    if (!selection || editorIsEmpty) {
        return [0, 0];
    }

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
 * stopAtNode? Exported for testing purposes
 */
export function textLength(node: Node, stopChildNumber: number): number {
    if (node.nodeType === Node.TEXT_NODE) {
        // for a text node, we may have to add an extra offset if it's inside a
        // certain container
        const shouldAddOffset = textNodeNeedsExtraOffset(node);
        const extraOffset = shouldAddOffset ? 1 : 0;
        return (node.textContent?.length ?? 0) + extraOffset;
    } else if (node.nodeName === 'BR') {
        // Treat br tags as being 1 character long, unless we are
        // looking for location 0 inside one, in which case it's 0 length

        return stopChildNumber === 0 ? 0 : 1;
    } else {
        // Add up lengths until we hit the stop node.
        let sum = 0;
        let i = 0;
        for (const ch of node.childNodes) {
            if (i === stopChildNumber) {
                break;
            }
            sum += textLength(ch, -1);
            i++;
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
    const isTextNode = currentNode.nodeType === Node.TEXT_NODE;
    const isInsideMention =
        currentNode.parentElement?.hasAttribute('data-mention-type');

    if (currentNode === nodeToFind) {
        // We've found the right node
        if (isTextNode) {
            // Text node - use the offset to know where we are
            if (offsetToFind > (currentNode.textContent?.length ?? 0)) {
                // If the offset is wrong, we didn't find it
                return { found: false, offset: 0 };
            } else {
                // Otherwise, we did

                // Special case for mention nodes which have length of 1
                if (isInsideMention) {
                    return { found: true, offset: offsetToFind === 0 ? 0 : 1 };
                }

                return { found: true, offset: offsetToFind };
            }
        } else {
            // Non-text node - offset is the index of the selected node
            // within currentNode.
            // Add up the sizes of all the nodes before offset.
            const ret = textLength(currentNode, offsetToFind);
            return { found: true, offset: ret };
        }
    } else {
        // We have not found the right node yet

        if (isTextNode) {
            // Return how many steps forward we progress by skipping
            // this node.

            // The extra check for an offset here depends on the ancestor of the
            // text node and can be seen as the opposite to the equivalent call
            // in computeNodeAndOffset
            const shouldAddOffset = textNodeNeedsExtraOffset(currentNode);
            const extraOffset = shouldAddOffset ? 1 : 0;

            // ...but also have a special case where we don't count a textnode
            // if it is an nbsp, as this is what we use to mark out empty
            // paragraphs
            if (isPlaceholderParagraphNode(currentNode)) {
                return { found: false, offset: extraOffset };
            }

            // ...and a special case where mentions alwayd have a length of 1
            if (isInsideMention) {
                return { found: false, offset: 1 + extraOffset };
            }

            return {
                found: false,
                offset: (currentNode.textContent?.length ?? 0) + extraOffset,
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
    // Special case - if asked for after the last node of the editor excluding
    // the line break tag (which we get if we do select-all), then we
    // return the end of the editor.
    if (node === editor && offset === editor.childNodes.length - 1) {
        return textLength(editor, -1) - 1;
    }

    // Check for before or after the editor itself
    const editorRange = new Range();
    editorRange.setStart(editor, 0);
    editorRange.setEnd(editor, editor.childNodes.length);
    const cmp = editorRange.comparePoint(node, 0);
    if (cmp === -1) {
        // Before the editor - count as at the beginning
        return 0;
    } else if (cmp === 1) {
        // After the editor - count as at the end
        return textLength(editor, -1) - 1;
    }

    const ret = findCharacter(editor, node, offset);
    if (ret.found) {
        return ret.offset;
    } else {
        return -1;
    }
}

/**
 * Given a text node, determine if we need to add an additional offset to it. A
 * text node that has any ancestor that is a li, pre, blockquote or p tag will
 * require an additional offset to match up with the rust model. We also need to
 * handle the case where we have consecutive formatting nodes, as only the last
 * of the formatting nodes needs the extra offset (if applicable).
 *
 * This is called in two places and only with TextNodes. Only exported for test.
 *
 * Returns a boolean, true if the node needs an extra offset
 */

export function textNodeNeedsExtraOffset(node: Node | null): boolean {
    if (node === null) return false;

    let checkNode: Node | ParentNode | null = node;
    const hasFormattingParent = isInlineNode(checkNode.parentNode);

    // If the parent is _not_ a formatting node, then we have a case where the
    // text node we are looking at is in a container, so we simply need to check
    // the ancestors to see if one of those containers requires an extra offset.

    // Otherwise we are dealing with the case where we are inside at least one
    // formatting node (but we could be deeper than that) and we also need to
    // make sure that we don't add the offset more than once when we have
    // multiple adjacent inline formatting nodes.
    while (checkNode) {
        // break the loop and return false if
        // either we find an inline node next
        // or we have a formatting ancestor and the next sibling is not
        // a container node
        // or the next node is a <br/> tag, as this is the end of the document
        const nextSibling = checkNode.nextSibling;
        if (
            (nextSibling && isInlineNode(nextSibling)) ||
            (hasFormattingParent &&
                nextSibling &&
                !isNodeRequiringExtraOffset(nextSibling)) ||
            (nextSibling && nextSibling.nodeName === 'BR')
        ) {
            break;
        }

        if (isNodeRequiringExtraOffset(checkNode)) {
            return true;
        } else {
            checkNode = checkNode.parentNode;
        }
    }
    return false;
}

const INLINE_NODE_NAMES = ['EM', 'U', 'STRONG', 'DEL', 'CODE', 'A'];
const EXTRA_OFFSET_NODE_NAMES = ['OL', 'UL', 'LI', 'PRE', 'BLOCKQUOTE', 'P'];

function isInlineNode(node: Node | ParentNode | null): boolean {
    if (node === null) return false;
    return INLINE_NODE_NAMES.includes(node.nodeName || '');
}

// Due to the way the rust model handles indexing, all of the nodes in the EXTRA
// _OFFSET_NODE_NAMES array will require the addition of an offset (of one).
// Nb, whilst lists require this offset, we consider the offset to apply to each
// list item, the enclosing list type tag does not add an extra offset.
// We need the enclosing list tags in the array as we also use this check on
// sibling nodes.
function isNodeRequiringExtraOffset(node: Node): boolean {
    return EXTRA_OFFSET_NODE_NAMES.includes(node.nodeName || '');
}

function isEmptyInlineNode(node: Node): boolean {
    return (
        INLINE_NODE_NAMES.includes(node.nodeName) &&
        node.textContent?.length === 0
    );
}

export function isPlaceholderParagraphNode(node: Node): boolean {
    // a placeholder paragraph will have single child that is a text node with
    // a content that is an nbsp
    const hasNoSiblings = node.parentNode?.childNodes.length === 1;
    const hasParagraphParent = node.parentNode?.nodeName === 'P';
    const hasNbspContent = node.textContent === String.fromCharCode(160);
    return hasParagraphParent && hasNoSiblings && hasNbspContent;
}
