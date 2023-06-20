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

import {
    computeNodeAndOffset,
    countCodeunit,
    getCurrentSelection,
    isPlaceholderParagraphNode,
    textLength,
    textNodeNeedsExtraOffset,
} from './dom';
let beforeEditor: HTMLDivElement;
let editor: HTMLDivElement;
let afterEditor: HTMLDivElement;

beforeAll(() => {
    beforeEditor = document.createElement('div');
    editor = document.createElement('div');
    editor.setAttribute('contentEditable', 'true');
    afterEditor = document.createElement('div');
    document.body.appendChild(beforeEditor);
    document.body.appendChild(editor);
    document.body.appendChild(afterEditor);
});

afterAll(() => {
    document.body.innerHTML = '';
});

describe('computeNodeAndOffset', () => {
    it('Should find at the start of simple text', () => {
        // When
        setEditorHtml('abcdefgh');
        const { node, offset } = computeNodeAndOffset(editor, 0);

        // Then
        expect(node).toBe(editor.childNodes[0]);
        expect(offset).toBe(0);
    });

    it('Should find in the middle of simple text', () => {
        // When
        setEditorHtml('abcdefgh');
        const { node, offset } = computeNodeAndOffset(editor, 4);

        // Then
        expect(node).toBe(editor.childNodes[0]);
        expect(offset).toBe(4);
    });

    it('Should find at the end of simple text', () => {
        // When
        setEditorHtml('abcdefgh');
        const { node, offset } = computeNodeAndOffset(editor, 8);

        // Then
        expect(node).toBe(editor.childNodes[0]);
        expect(offset).toBe(8);
    });

    it('Should return null if off the end', () => {
        // When
        setEditorHtml('abcdefgh');
        // 8 characters, plus the br we always append = 9, so 10 is off end
        const { node, offset } = computeNodeAndOffset(editor, 10);

        // Then
        expect(node).toBeNull();
        expect(offset).toBe(1);
    });

    it('Should find before subnode', () => {
        // When
        setEditorHtml('abc<b>def</b>gh');
        const { node, offset } = computeNodeAndOffset(editor, 2);

        // Then
        expect(node).toBe(editor.childNodes[0]);
        expect(offset).toBe(2);
    });

    it('Should find after subnode', () => {
        // When
        setEditorHtml('abc<b>def</b>gh');
        const { node, offset } = computeNodeAndOffset(editor, 4);

        // Then
        expect(node).toBe(editor.childNodes[1].childNodes[0]);
        expect(offset).toBe(1);
    });

    it('Should find inside subnode', () => {
        // When
        setEditorHtml('abc<b>def</b>gh');
        const { node, offset } = computeNodeAndOffset(editor, 7);

        // Then
        expect(node).toBe(editor.childNodes[2]);
        expect(offset).toBe(1);
    });

    it('Should find after subnode', () => {
        // When
        setEditorHtml('abc<b>def</b>gh');
        const { node, offset } = computeNodeAndOffset(editor, 7);

        // Then
        expect(node).toBe(editor.childNodes[2]);
        expect(offset).toBe(1);
    });

    it('Should find before br', () => {
        // When
        setEditorHtml('a<br />b');
        const { node, offset } = computeNodeAndOffset(editor, 0);

        // Then
        expect(node).toBe(editor.childNodes[0]);
        expect(offset).toBe(0);
    });

    it('Should find br start', () => {
        // When
        setEditorHtml('a<br />b');
        const { node, offset } = computeNodeAndOffset(editor, 1);

        // Then
        expect(node).toBe(editor.childNodes[0]);
        expect(offset).toBe(1);
    });

    it('Should find br end', () => {
        // When
        setEditorHtml('a<br />b');
        const { node, offset } = computeNodeAndOffset(editor, 2);

        // Then
        expect(node).toBe(editor.childNodes[2]);
        expect(offset).toBe(0);
    });

    it('Should find between br', () => {
        // When
        setEditorHtml('a<br /><br />b');
        const { node, offset } = computeNodeAndOffset(editor, 2);

        // Then
        expect(node).toBe(editor.childNodes[2]);
        expect(offset).toBe(0);
    });

    it('Should find br at end', () => {
        // When
        setEditorHtml('abc<br />');
        const { node, offset } = computeNodeAndOffset(editor, 4);

        // Then
        expect(node).toBe(editor.childNodes[2]);
        expect(offset).toBe(0);
    });

    it('Should find after br', () => {
        // When
        setEditorHtml('a<br />b');
        const { node, offset } = computeNodeAndOffset(editor, 3);

        // Then
        expect(node).toBe(editor.childNodes[2]);
        expect(offset).toBe(1);
    });

    it('Should find inside a paragraph', () => {
        // When
        setEditorHtml('<p>a</p>');
        const { node: startNode, offset: startOffset } = computeNodeAndOffset(
            editor,
            0,
        );
        const { node: endNode, offset: endOffset } = computeNodeAndOffset(
            editor,
            1,
        );

        // Then
        expect(startNode).toBe(editor.childNodes[0].childNodes[0]);
        expect(startOffset).toBe(0);

        expect(endNode).toBe(editor.childNodes[0].childNodes[0]);
        expect(endOffset).toBe(1);
    });

    it('Should find inside adjacent paragraphs', () => {
        // When
        setEditorHtml('<p>a</p><p>b</p>');
        const { node: fistChildNode, offset: firstChildOffset } =
            computeNodeAndOffset(editor, 1);
        const { node: secondChildNode, offset: secondChildOffset } =
            computeNodeAndOffset(editor, 2);

        // Then
        expect(fistChildNode).toBe(editor.childNodes[0].childNodes[0]);
        expect(firstChildOffset).toBe(1);

        expect(secondChildNode).toBe(editor.childNodes[1].childNodes[0]);
        expect(secondChildOffset).toBe(0);
    });

    it('Should find inside adjacent nested paragraphs, first child', () => {
        // When
        setEditorHtml('<p><em>a</em></p><p><em>b</em></p>');
        const { node: firstChildNode, offset: firstChildOffset } =
            computeNodeAndOffset(editor, 1);
        const { node: secondChildNode, offset: secondChildOffset } =
            computeNodeAndOffset(editor, 2);

        // Then
        expect(firstChildNode).toBe(
            editor.childNodes[0].childNodes[0].childNodes[0],
        );
        expect(firstChildOffset).toBe(1);

        expect(secondChildNode).toBe(
            editor.childNodes[1].childNodes[0].childNodes[0],
        );
        expect(secondChildOffset).toBe(0);
    });

    it('Should find inside adjacent empty paragraph, second child', () => {
        // When
        // we get this when we start writing in the composer (goes in as plain
        // text) and then we press enter and we move to paragraphs
        setEditorHtml('<p>press enter</p><p>&nbsp;</p>');
        const { node, offset } = computeNodeAndOffset(editor, 12);

        // Then
        expect(node).toBe(editor.childNodes[1].childNodes[0]);
        expect(offset).toBe(0);
    });

    it('Should find inside an empty list', () => {
        // When
        setEditorHtml('<ul><li></li></ul>');
        const { node, offset } = computeNodeAndOffset(editor, 0);

        // Then
        expect(node).toBe(editor.childNodes[0].childNodes[0]);
        expect(offset).toBe(0);
    });

    it('Should find inside an empty single list item', () => {
        // When
        setEditorHtml('<ul><li></li></ul>');
        const { node, offset } = computeNodeAndOffset(editor, 0);

        // Then
        expect(node).toBe(editor.childNodes[0].childNodes[0]);
        expect(offset).toBe(0);
    });

    it('Should find inside a single list item', () => {
        // When
        setEditorHtml('<ul><li>foo</li></ul>');
        const { node, offset } = computeNodeAndOffset(editor, 1);

        // Then
        expect(node).toBe(editor.childNodes[0].childNodes[0].childNodes[0]);
        expect(offset).toBe(1);
    });

    it('Should find inside children of empty list items', () => {
        // When
        setEditorHtml('<ul><li></li><li></li></ul>');
        const { node: firstChildNode, offset: firstChildOffset } =
            computeNodeAndOffset(editor, 0);

        // Then
        expect(firstChildNode).toBe(editor.childNodes[0].childNodes[0]);
        expect(firstChildOffset).toBe(0);
    });

    it('Should find inside adjacent list items', () => {
        // When
        setEditorHtml('<ul><li>foo</li><li>bar</li></ul>');
        const { node: firstChildNode, offset: firstChildOffset } =
            computeNodeAndOffset(editor, 3);
        const { node: secondChildNode, offset: secondChildOffset } =
            computeNodeAndOffset(editor, 4);

        // Then
        expect(firstChildNode).toBe(
            editor.childNodes[0].childNodes[0].childNodes[0],
        );
        expect(firstChildOffset).toBe(3);

        expect(secondChildNode).toBe(
            editor.childNodes[0].childNodes[1].childNodes[0],
        );
        expect(secondChildOffset).toBe(0);
    });

    it('Should find inside adjacent lists', () => {
        // When
        setEditorHtml('<ul><li>foo</li></ul><ul><li>bar</li></ul>');
        const { node: firstListNode, offset: firstListOffset } =
            computeNodeAndOffset(editor, 3);
        const { node: secondListNode, offset: secondListOffset } =
            computeNodeAndOffset(editor, 4);

        // Then
        expect(firstListNode).toBe(
            editor.childNodes[0].childNodes[0].childNodes[0],
        );
        expect(firstListOffset).toBe(3);

        expect(secondListNode).toBe(
            editor.childNodes[1].childNodes[0].childNodes[0],
        );
        expect(secondListOffset).toBe(0);
    });

    it('Should find inside quote', () => {
        // When
        setEditorHtml('<blockquote>quote</blockquote>');
        const { node, offset } = computeNodeAndOffset(editor, 2);

        // Then
        expect(node).toBe(editor.childNodes[0].childNodes[0]);
        expect(offset).toBe(2);
    });

    it('Should find inside quote followed by another container node', () => {
        // When
        const firstNode = '<blockquote>quote</blockquote>';
        const nextOtherNodes = [
            '<ol><li>ordered list</li></ol>',
            '<ul><li>ordered list</li></ul>',
            '<p>paragraph</p>',
            '<pre><code>codeblock</code></pre>',
            '<blockquote>another quote</blockquote>',
        ];

        nextOtherNodes.forEach((nextNode) => {
            setEditorHtml(firstNode + nextNode);
            const { node, offset } = computeNodeAndOffset(editor, 5);

            // Then
            expect(node).toBe(editor.childNodes[0].childNodes[0]);
            expect(offset).toBe(5);
        });
    });

    it('Should find inside quote container node after a quote', () => {
        // When
        const firstNode = '<blockquote>quote</blockquote>';
        const nextNodes = [
            { node: '<ol><li>ordered list</li></ol>', depth: 2 },
            { node: '<ul><li>ordered list</li></ul>', depth: 2 },
            { node: '<pre><code>codeblock</code></pre>', depth: 2 },
            { node: '<p>paragraph</p>', depth: 1 },
            { node: '<blockquote>another quote</blockquote>', depth: 1 },
        ];

        nextNodes.forEach((nextNode) => {
            setEditorHtml(firstNode + nextNode.node);
            const { node, offset } = computeNodeAndOffset(editor, 6);

            let editorNextNode = editor.childNodes[1];
            for (let i = 0; i < nextNode.depth; i++) {
                editorNextNode = editorNextNode.childNodes[0];
            }
            // Then
            expect(node).toBe(editorNextNode);
            expect(offset).toBe(0);
        });
    });

    it('Should count newlines as characters in code blocks', () => {
        // When
        setEditorHtml('<pre><code>length\nis 12</code></pre>');
        const { node } = computeNodeAndOffset(editor, 5);

        // Then
        expect(node).toBe(editor.childNodes[0].childNodes[0].childNodes[0]);
        expect(node?.textContent).toHaveLength(12);
    });

    // eslint-disable-next-line max-len
    it('does not count the length of "empty paragraphs" ie paragraphs with an nbsp inside them', () => {
        // When
        setEditorHtml('<p><del>foo</del></p><p>&nbsp;</p><p><del></del></p>');
        const { node, offset } = computeNodeAndOffset(editor, 5);

        // Then
        expect(node).toBe(editor.childNodes[2].childNodes[0]);
        expect(offset).toBe(0);
    });

    it('allows multiple empty p tags and works as expected', () => {
        // When
        setEditorHtml(
            '<p><del>foo</del></p><p>&nbsp;</p><p>&nbsp;</p><p><del></del></p>',
        );
        const { node, offset } = computeNodeAndOffset(editor, 6);

        // Then
        expect(node).toBe(editor.childNodes[3].childNodes[0]);
        expect(offset).toBe(0);
    });

    // eslint-disable-next-line max-len
    it('should deal with nbsp caused by line breaking part way through a tag', () => {
        // When
        // case when we have <strong>bold</strong> line2 then move cursor to
        // just before the l and press enter
        setEditorHtml('<p><strong>bold</strong>&nbsp;</p><p>line2</p>');
        const { node, offset } = computeNodeAndOffset(editor, 6);

        // Then
        expect(node).toBe(editor.childNodes[1].childNodes[0]);
        expect(offset).toBe(0);
    });

    // TODO remove attributes from mentions when Rust model can parse url
    // https://github.com/matrix-org/matrix-rich-text-editor/issues/709
    it('can find the beginning of a mention correctly', () => {
        // When
        setEditorHtml(
            // eslint-disable-next-line max-len
            '<a data-mention-type="something" contenteditable="false">test</a>&nbsp;',
        );
        const { node, offset } = computeNodeAndOffset(editor, 0);

        // Then
        expect(node).toBe(editor);
        expect(offset).toBe(0);
    });

    it('can find the end of a mention correctly', () => {
        // When
        // we have a mention, ie a tag with a data-mention-type attribute
        setEditorHtml(
            // eslint-disable-next-line max-len
            '<a data-mention-type="something" contenteditable="false">test</a>&nbsp;',
        );
        const { node, offset } = computeNodeAndOffset(editor, 1);

        // Then
        expect(node).toBe(editor.childNodes[1]);
        expect(offset).toBe(0);
    });

    it('can find the nbsp after a mention correctly', () => {
        // When
        // we have a mention, ie a tag with a data-mention-type attribute
        setEditorHtml(
            // eslint-disable-next-line max-len
            '<a data-mention-type="something" contenteditable="false">test</a>&nbsp;',
        );
        const { node, offset } = computeNodeAndOffset(editor, 2);

        // Then
        expect(node).toBe(editor.childNodes[1]);
        expect(offset).toBe(1);
    });

    it('can find the beginning of a mention inside a paragraph', () => {
        // When
        setEditorHtml(
            // eslint-disable-next-line max-len
            '<p><a data-mention-type="something" contenteditable="false">test</a>&nbsp;</p>',
        );
        const { node, offset } = computeNodeAndOffset(editor, 0);

        // Then
        expect(node).toBe(editor.childNodes[0]);
        expect(offset).toBe(0);
    });

    it('can find the start of nbsp after a mention inside a paragraph', () => {
        // When
        setEditorHtml(
            // eslint-disable-next-line max-len
            '<p><a data-mention-type="something" contenteditable="false">test</a>&nbsp;</p>',
        );
        const { node, offset } = computeNodeAndOffset(editor, 1);

        // Then
        expect(node).toBe(editor.childNodes[0].childNodes[1]);
        expect(offset).toBe(0);
    });

    it('can find the end of nbsp after a mention inside a paragraph', () => {
        // When
        setEditorHtml(
            // eslint-disable-next-line max-len
            '<p><a data-mention-type="something" contenteditable="false">test</a>&nbsp;</p>',
        );
        const { node, offset } = computeNodeAndOffset(editor, 2);

        // Then
        expect(node).toBe(editor.childNodes[0].childNodes[1]);
        expect(offset).toBe(1);
    });
});

describe('countCodeunit', () => {
    it('Returns the end of the editor when whole editor selected', () => {
        const plainString = 'abcdefgh';
        const htmlString = '<p>text in</p><p>paragraph tags</p>';

        setEditorHtml(plainString);
        expect(countCodeunit(editor, editor, 1)).toBe(plainString.length);

        setEditorHtml(htmlString);
        expect(countCodeunit(editor, editor, 2)).toBe(22);
    });
    it('Should count ASCII', () => {
        // When
        setEditorHtml('abcdefgh');
        const textNode = editor.childNodes[0];

        // Then
        expect(countCodeunit(editor, textNode, 0)).toBe(0);
        expect(countCodeunit(editor, textNode, 3)).toBe(3);
        expect(countCodeunit(editor, textNode, 7)).toBe(7);
        // Just past the end is allowed
        expect(countCodeunit(editor, textNode, 8)).toBe(8);
        // But not past that
        expect(countCodeunit(editor, textNode, 9)).toBe(-1);
    });

    it('Should count UCS-2', () => {
        // When
        setEditorHtml('a\u{03A9}b\u{03A9}c');
        const textNode = editor.childNodes[0];

        // Then
        expect(countCodeunit(editor, textNode, 0)).toBe(0);
        expect(countCodeunit(editor, textNode, 1)).toBe(1);
        expect(countCodeunit(editor, textNode, 4)).toBe(4);
        expect(countCodeunit(editor, textNode, 5)).toBe(5);
        expect(countCodeunit(editor, textNode, 6)).toBe(-1);
    });

    it('Should count complex', () => {
        // When
        setEditorHtml('a\u{1F469}\u{1F3FF}\u{200D}\u{1F680}b');
        const textNode = editor.childNodes[0];

        // Then
        expect(countCodeunit(editor, textNode, 0)).toBe(0);
        expect(countCodeunit(editor, textNode, 7)).toBe(7);
        expect(countCodeunit(editor, textNode, 8)).toBe(8);
        expect(countCodeunit(editor, textNode, 9)).toBe(9);
        expect(countCodeunit(editor, textNode, 10)).toBe(-1);
    });

    it('Should count nested', () => {
        // When
        setEditorHtml('a<b>b</b>c');
        const firstTextNode = editor.childNodes[0];
        const boldTextNode = editor.childNodes[1].childNodes[0];
        const thirdTextNode = editor.childNodes[2];

        // Then
        expect(countCodeunit(editor, firstTextNode, 0)).toBe(0);
        expect(countCodeunit(editor, boldTextNode, 0)).toBe(1);
        expect(countCodeunit(editor, thirdTextNode, 0)).toBe(2);
    });

    it('Should treat br as a character', () => {
        // When
        setEditorHtml('a<br />b');
        const firstTextNode = editor.childNodes[0];
        const brNode = editor.childNodes[1];
        const secondTextNode = editor.childNodes[2];

        // Then
        expect(countCodeunit(editor, firstTextNode, 0)).toBe(0);
        expect(countCodeunit(editor, brNode, 1)).toBe(2);
        expect(countCodeunit(editor, secondTextNode, 1)).toBe(3);
    });

    it('Should work with deeply nested', () => {
        // When
        setEditorHtml('aaa<b><i>bbb</i>ccc</b>ddd');
        const firstTextNode = editor.childNodes[0];
        const boldItalicTextNode =
            editor.childNodes[1].childNodes[0].childNodes[0];
        const boldOnlyTextNode = editor.childNodes[1].childNodes[1];
        const thirdTextNode = editor.childNodes[2];

        // Then
        expect(countCodeunit(editor, firstTextNode, 1)).toBe(1);
        expect(countCodeunit(editor, firstTextNode, 2)).toBe(2);
        expect(countCodeunit(editor, firstTextNode, 3)).toBe(3);
        expect(countCodeunit(editor, boldItalicTextNode, 0)).toBe(3);
        expect(countCodeunit(editor, boldItalicTextNode, 1)).toBe(4);
        expect(countCodeunit(editor, boldItalicTextNode, 2)).toBe(5);
        expect(countCodeunit(editor, boldOnlyTextNode, 0)).toBe(6);
        expect(countCodeunit(editor, boldOnlyTextNode, 1)).toBe(7);
        expect(countCodeunit(editor, boldOnlyTextNode, 2)).toBe(8);
        expect(countCodeunit(editor, thirdTextNode, 0)).toBe(9);
        expect(countCodeunit(editor, thirdTextNode, 1)).toBe(10);
        expect(countCodeunit(editor, thirdTextNode, 2)).toBe(11);
    });

    it('Should count mentions as having length 1', () => {
        // When
        // we use the presence of a data-mention-type attribute to determine
        // if we have a mention, the tag is unimportant
        setEditorHtml(
            // eslint-disable-next-line max-len
            'hello <span data-mention-type="something" contenteditable="false">Alice</span>',
        );
        const helloTextNode = editor.childNodes[0];
        const mentionTextNode = editor.childNodes[1].childNodes[0];

        // Then
        expect(countCodeunit(editor, helloTextNode, 0)).toBe(0);
        expect(countCodeunit(editor, helloTextNode, 6)).toBe(6);
        expect(countCodeunit(editor, mentionTextNode, 0)).toBe(6);
        expect(countCodeunit(editor, mentionTextNode, 1)).toBe(7);
        expect(countCodeunit(editor, mentionTextNode, 2)).toBe(7);
        expect(countCodeunit(editor, mentionTextNode, 3)).toBe(7);
        expect(countCodeunit(editor, mentionTextNode, 4)).toBe(7);
        expect(countCodeunit(editor, mentionTextNode, 5)).toBe(7);
    });
});

describe('getCurrentSelection', () => {
    it('correctly locates the cursor in an empty editor', () => {
        setEditorHtml('');
        const sel = selectAll();
        expect(getCurrentSelection(editor, sel)).toEqual([0, 0]);
    });

    it('correctly locates the cursor in adjacent paragraphs', () => {
        setEditorHtml('<p>para 1</p><p>para 2</p>');
        const firstParagraphTextNode = editor.childNodes[0].childNodes[0];
        const secondParagrahTextNode = editor.childNodes[1].childNodes[0];
        const firstNodeOffset = 4;
        const secondNodeOffset = 1;

        // test the first paragraph
        let sel = putCaretInTextNodeAtOffset(
            firstParagraphTextNode,
            firstNodeOffset,
        );

        // Sanity: the focusNode and anchorNode are the first text node
        // and the offset tells you how far into that text node we are
        expect(sel.anchorNode).toBe(firstParagraphTextNode);
        expect(sel.anchorOffset).toBe(firstNodeOffset);
        expect(sel.focusNode).toBe(firstParagraphTextNode);
        expect(sel.focusOffset).toBe(firstNodeOffset);

        // We should see ourselves as on code unit firstNodeOffset
        expect(getCurrentSelection(editor, sel)).toEqual([
            firstNodeOffset,
            firstNodeOffset,
        ]);

        // move to the second paragraph
        sel = putCaretInTextNodeAtOffset(
            secondParagrahTextNode,
            secondNodeOffset,
        );

        // Sanity: the focusNode and anchorNode are the second text node
        // and the offset tells you how far into that text node we are
        expect(sel.anchorNode).toBe(secondParagrahTextNode);
        expect(sel.anchorOffset).toBe(secondNodeOffset);
        expect(sel.focusNode).toBe(secondParagrahTextNode);
        expect(sel.focusOffset).toBe(secondNodeOffset);

        // We should see ourselves as on code unit 8, because a paragraph tag
        // will add an extra offset, so our total offset is the length of the
        // first paragraph, plus the extra offset, plus the second node offset
        // ie 6 + 1 + 1 = 8
        expect(getCurrentSelection(editor, sel)).toEqual([8, 8]);
    });

    it('correctly locates the cursor inside nested tags', () => {
        setEditorHtml(
            '<p>pa<strong>ra 1</strong></p><p><strong>pa</strong>ra 2</p>',
        );
        const firstParagraphStrongTextNode =
            editor.childNodes[0].childNodes[1].childNodes[0];
        const secondParagrahStrongTextNode =
            editor.childNodes[1].childNodes[0].childNodes[0];
        const firstNodeOffset = 0;
        const secondNodeOffset = 2;

        // test the first paragraph strong node
        let sel = putCaretInTextNodeAtOffset(
            firstParagraphStrongTextNode,
            firstNodeOffset,
        );

        // Sanity: the focusNode and anchorNode are the first strong text node
        // and the offset tells you how far into that text node we are
        expect(sel.anchorNode).toBe(firstParagraphStrongTextNode);
        expect(sel.anchorOffset).toBe(firstNodeOffset);
        expect(sel.focusNode).toBe(firstParagraphStrongTextNode);
        expect(sel.focusOffset).toBe(firstNodeOffset);

        // Sanity: the focusNode and anchorNode are the first strong text node
        // and the offset tells you how far into that text node we are
        expect(sel.anchorNode).toBe(firstParagraphStrongTextNode);
        expect(sel.anchorOffset).toBe(firstNodeOffset);
        expect(sel.focusNode).toBe(firstParagraphStrongTextNode);
        expect(sel.focusOffset).toBe(firstNodeOffset);

        // move to the second paragraph
        sel = putCaretInTextNodeAtOffset(
            secondParagrahStrongTextNode,
            secondNodeOffset,
        );

        // Sanity: the focusNode and anchorNode are the second text node
        // and the offset tells you how far into that text node we are
        expect(sel.anchorNode).toBe(secondParagrahStrongTextNode);
        expect(sel.anchorOffset).toBe(secondNodeOffset);
        expect(sel.focusNode).toBe(secondParagrahStrongTextNode);
        expect(sel.focusOffset).toBe(secondNodeOffset);

        // We should see ourselves as on code unit 9, because a paragraph tag
        // will add an extra offset, so our total offset is the length of the
        // first paragraph, plus the extra offset, plus the second node offset
        // ie 6 + 1 + 2 = 8
        expect(getCurrentSelection(editor, sel)).toEqual([9, 9]);
    });

    it('correctly finds backward selections in adjacent paragraphs', () => {
        setEditorHtml('<p>para 1</p><p>para 2</p>');
        const firstParagraphTextNode = editor.childNodes[0].childNodes[0];
        const firstOffset = 4;
        const secondParagraphTextNode = editor.childNodes[1].childNodes[0];
        const secondNodeOffset = 6;

        const sel = putCaretInTextNodeAtOffset(
            secondParagraphTextNode,
            secondNodeOffset,
        );
        sel.extend(firstParagraphTextNode, firstOffset);

        // Sanity: the anchorNode is where we started, in the second text node,
        // and the focusNode is where we moved to, the first text node
        expect(sel.anchorNode).toBe(secondParagraphTextNode);
        expect(sel.anchorOffset).toBe(secondNodeOffset);
        expect(sel.focusNode).toBe(firstParagraphTextNode);
        expect(sel.focusOffset).toBe(firstOffset);

        expect(getCurrentSelection(editor, sel)).toEqual([13, 4]);
    });

    it('handles selecting all with ctrl-a', () => {
        setEditorHtml('<p>para 1</p><p>para 2</p>');
        const sel = selectAll();
        expect(getCurrentSelection(editor, sel)).toEqual([0, 13]);
    });

    it('handles selecting all by dragging from start to end', () => {
        setEditorHtml('<p>para 1</p><p>para 2</p>');
        const firstParagraphTextNode = editor.childNodes[0].childNodes[0];
        const firstOffset = 0;
        const secondParagraphTextNode = editor.childNodes[1].childNodes[0];
        const secondNodeOffset = 6;

        const sel = putCaretInTextNodeAtOffset(
            firstParagraphTextNode,
            firstOffset,
        );
        sel.extend(secondParagraphTextNode, secondNodeOffset);

        expect(getCurrentSelection(editor, sel)).toEqual([0, 13]);
    });

    it('handles selecting all by dragging backwards from end to start', () => {
        setEditorHtml('<p>para 1</p><p>para 2</p>');
        const firstParagraphTextNode = editor.childNodes[0].childNodes[0];
        const firstOffset = 0;
        const secondParagraphTextNode = editor.childNodes[1].childNodes[0];
        const secondNodeOffset = 6;

        const sel = putCaretInTextNodeAtOffset(
            secondParagraphTextNode,
            secondNodeOffset,
        );
        sel.extend(firstParagraphTextNode, firstOffset);

        expect(getCurrentSelection(editor, sel)).toEqual([13, 0]);
    });

    it('handles selecting across multiple newlines', () => {
        setEditorHtml('<p>para 1</p><p>para 2</p>');
        const firstParagraphTextNode = editor.childNodes[0].childNodes[0];
        const firstOffset = 2;
        const secondParagraphTextNode = editor.childNodes[1].childNodes[0];
        const secondNodeOffset = 2;

        const sel = putCaretInTextNodeAtOffset(
            firstParagraphTextNode,
            firstOffset,
        );
        sel.extend(secondParagraphTextNode, secondNodeOffset);

        expect(getCurrentSelection(editor, sel)).toEqual([2, 9]);
    });

    it('handles cursor after end', () => {
        setEditorHtml('<p>para 1</p><p>para 2</p>');
        // Simulate going to end of doc and pressing down arrow
        const sel = cursorToAfterEnd();
        expect(getCurrentSelection(editor, sel)).toEqual([13, 13]);
    });

    it('handles cursor at start', () => {
        setEditorHtml('<p>para 1</p><p>para 2</p>');
        const sel = cursorToBeginning();
        expect(getCurrentSelection(editor, sel)).toEqual([0, 0]);
    });

    it('handles selection before the start by returning 0, 0', () => {
        setEditorHtml('<p>para 1</p><p>para 2</p>');
        const sel = selectionOutsideEditor('before');
        expect(getCurrentSelection(editor, sel)).toEqual([0, 0]);
    });

    it('handles selection after the end by returning last character', () => {
        setEditorHtml('<p>para 1</p><p>para 2</p>');
        const sel = selectionOutsideEditor('after');
        expect(getCurrentSelection(editor, sel)).toEqual([13, 13]);
    });
});

describe('textNodeNeedsExtraOffset', () => {
    const trueTestCases = [
        { name: 'paragraph', testTag: 'p' },
        { name: 'preformatted', testTag: 'pre' },
        { name: 'block quote', testTag: 'blockquote' },
        { name: 'list item (ordered)', testTag: 'li', wrappingTag: 'ol' },
        { name: 'list item (unordered)', testTag: 'li', wrappingTag: 'ul' },
    ];

    const falseTestCases = [
        { name: 'bold', testTag: 'strong' },
        { name: 'italic', testTag: 'em' },
        { name: 'underline', testTag: 'u' },
        { name: 'strikethrough', testTag: 'del' },
        { name: 'link', testTag: 'a' },
        { name: 'inline code', testTag: 'code' },
    ];

    it.each(trueTestCases)(
        'Should add an offset to text node inside ${name} tag',
        ({ testTag, wrappingTag = '' }) => {
            // When
            // to allow us to handle list items in the test cases we need a way
            // to calculate the wrapping tag if required
            const openingTag = wrappingTag ? `<${wrappingTag}>` : '';
            const closingTag = wrappingTag ? `<${wrappingTag}>` : '';
            setEditorHtml(
                // eslint-disable-next-line max-len
                `${openingTag}<${testTag}>test test</${testTag}>${closingTag}<p>some adjacent text</p>`,
            );
            const { node } = computeNodeAndOffset(editor, 1);

            // Then
            // to account for extra nesting in some cases
            if (wrappingTag) {
                expect(node).toBe(
                    editor.childNodes[0].childNodes[0].childNodes[0],
                );
            } else {
                expect(node).toBe(editor.childNodes[0].childNodes[0]);
            }

            expect(textNodeNeedsExtraOffset(node)).toBe(true);
        },
    );

    it.each(falseTestCases)(
        'Should not add an offset to text node inside ${name} tag',
        ({ testTag }) => {
            // When
            setEditorHtml(`<${testTag}>test test</${testTag}>`);
            const { node } = computeNodeAndOffset(editor, 1);

            // Then
            expect(node).toBe(editor.childNodes[0].childNodes[0]);
            expect(textNodeNeedsExtraOffset(node)).toBe(false);
        },
    );

    it('only applies offset to last of consecutive inline items', () => {
        // When
        setEditorHtml(`<p><u>test</u><em> words</em></p><p></p>`);
        // look at the `test` text node
        const { node: testNode } = computeNodeAndOffset(editor, 0);

        // Then
        expect(testNode).toBe(editor.childNodes[0].childNodes[0].childNodes[0]);
        expect(textNodeNeedsExtraOffset(testNode)).toBe(false);

        // look at the `words` text node
        const { node: wordsNode } = computeNodeAndOffset(editor, 5);

        expect(wordsNode).toBe(
            editor.childNodes[0].childNodes[1].childNodes[0],
        );
        expect(textNodeNeedsExtraOffset(wordsNode)).toBe(true);
    });

    it('can handle formatting inside list items with nested formatting', () => {
        // When
        setEditorHtml('<ol><li>reg <strong>b</strong></li></ol>');
        const { node } = computeNodeAndOffset(editor, 0);

        // Then
        expect(node).toBe(editor.childNodes[0].childNodes[0].childNodes[0]);
        expect(textNodeNeedsExtraOffset(node)).toBe(false);
    });

    it('does not apply offset to the last child node', () => {
        // When
        setEditorHtml('<p>single child</p>');

        const { node } = computeNodeAndOffset(editor, 0);

        // Then
        expect(node).toBe(editor.childNodes[0].childNodes[0]);
        expect(textNodeNeedsExtraOffset(node)).toBe(false);
    });

    // eslint-disable-next-line max-len
    it('applies extra offset for nested formatting inside a paragraph adjacent to unordered list', () => {
        // When
        setEditorHtml(
            '<p><strong>line one</strong></p><ul><li>item one</li></ul>',
        );

        // put the cursor before the "l" of line
        const { node } = computeNodeAndOffset(editor, 0);

        // Then
        // check that the node we have is the text node "line one" and that we
        // will add an offset for it
        expect(node).toBe(editor.childNodes[0].childNodes[0].childNodes[0]);
        expect(textNodeNeedsExtraOffset(node)).toBe(true);
    });

    // eslint-disable-next-line max-len
    it('applies extra offset for nested formatting inside a paragraph adjacent to unordered list', () => {
        // When
        setEditorHtml(
            '<p><strong>line one</strong></p><ol><li>item one</li></ol>',
        );

        // put the cursor before the "l" of line
        const { node } = computeNodeAndOffset(editor, 0);

        // Then
        // check that the node we have is the text node "line one" and that we
        // will add an offset for it
        expect(node).toBe(editor.childNodes[0].childNodes[0].childNodes[0]);
        expect(textNodeNeedsExtraOffset(node)).toBe(true);
    });
});

describe('textLength', () => {
    const testString = 'string for testing';
    const testStringLength = testString.length;
    it('calculates the length of a plain string inside a div correcly', () => {
        // this represents when the user initially starts typing, before any
        // paragraphs have been added
        const divNode = document.createElement('div');
        divNode.textContent = testString;
        expect(textLength(divNode, -1)).toBe(testStringLength);
    });

    it('calculates the length of a string inside a paragraph correctly', () => {
        // when we have a string inside a paragraph, the length needs to be
        // given an extra offset to match up with the rust model indices
        const paragraphNode = document.createElement('p');
        paragraphNode.textContent = testString;
        expect(textLength(paragraphNode, -1)).toBe(testStringLength + 1);
    });

    it('calculates the length of a strings inside a list correctly', () => {
        // when we have a list item inside a list, check that we only apply the
        // offset to the list items, not to the list container as well
        const listNode = document.createElement('ul');
        const numberOfListItems = 3;

        for (let i = 0; i < numberOfListItems; i++) {
            const listItem = document.createElement('li');
            listItem.textContent = testString;
            listNode.appendChild(listItem);
        }

        expect(textLength(listNode, -1)).toBe(
            numberOfListItems * (testStringLength + 1),
        );
    });
});

describe('isPlaceholderParagraphNode', () => {
    it('returns true for placeholder paragraph', () => {
        setEditorHtml('<p>&nbsp;</p>');
        const node = editor.childNodes[0].childNodes[0];
        expect(isPlaceholderParagraphNode(node)).toBe(true);
    });

    it('returns false for whitespace paragraph', () => {
        setEditorHtml('<p> </p>');
        const node = editor.childNodes[0].childNodes[0];
        expect(isPlaceholderParagraphNode(node)).toBe(false);
    });

    it('returns false for node with sibling', () => {
        setEditorHtml('<p>text<em>italic</em></p>');
        const node = editor.childNodes[0].childNodes[0];
        expect(isPlaceholderParagraphNode(node)).toBe(false);
    });

    it('returns false for node with non paragraph parent', () => {
        setEditorHtml('<div>text</div>');
        const node = editor.childNodes[0].childNodes[0];
        expect(isPlaceholderParagraphNode(node)).toBe(false);
    });
});

/* HELPER FUNCTIONS */
/* The editor always needs an extra BR after your HTML */
function setEditorHtml(html: string) {
    editor.innerHTML = html + '<br />';
}

/* Given a text node, place the a caret (ie zero length selection) 
at the given offset */
function putCaretInTextNodeAtOffset(node: Node, offset: number): Selection {
    if (node.nodeName !== '#text') {
        throw new Error(
            'Called putCaretInTextNodeAtOffset with a non-text node',
        );
    }

    // nb typing here is a little strange, we will only get a null back if
    // this is called on an iFrame with display:none from Firefox
    // ref: https://developer.mozilla.org/en-US/docs/Web/API/Window/getSelection#return_value
    const selection = document.getSelection();

    // clear out the selection and then set base and extent
    selection?.removeAllRanges();
    selection?.setBaseAndExtent(node, offset, node, offset);

    if (selection === null) {
        throw new Error('null selection created in putCaretInTextNodeAtOffset');
    }

    return selection;
}

/* Press cmd/ctrl + a */
function selectAll(): Selection | null {
    // select all works in the browser by selecting the first text node as
    // the anchor with an offset of 0, and the focus node as the editor,
    // with the offset equal to the number of children nodes - 1, to exclude
    // the last linebreak tag
    const selection = document.getSelection();
    selection?.removeAllRanges();

    const firstTextNode = document
        .createNodeIterator(editor, NodeFilter.SHOW_TEXT)
        .nextNode();

    if (firstTextNode) {
        selection?.setBaseAndExtent(
            firstTextNode,
            0,
            editor,
            editor.childNodes.length - 1, // ignore the final linebreak
        );
    }

    return selection;
}

/* Click at the end then press down arrow */
function cursorToAfterEnd(): Selection | null {
    const selection = document.getSelection();
    const offset = editor.childNodes.length - 1;
    selection?.setBaseAndExtent(editor, offset, editor, offset);
    return selection;
}

/** Click at the beginning */
function cursorToBeginning(): Selection | null {
    const selection = document.getSelection();
    const firstTextNode = document
        .createNodeIterator(editor, NodeFilter.SHOW_TEXT)
        .nextNode();

    if (firstTextNode) {
        selection?.setBaseAndExtent(firstTextNode, 0, firstTextNode, 0);
    }

    return selection;
}

/* Like selecting something before or after the editor */
function selectionOutsideEditor(
    location: 'before' | 'after',
): Selection | null {
    const selection = document.getSelection();
    const targetElement = location === 'before' ? beforeEditor : afterEditor;
    selection?.setBaseAndExtent(targetElement, 0, targetElement, 0);
    return selection;
}
