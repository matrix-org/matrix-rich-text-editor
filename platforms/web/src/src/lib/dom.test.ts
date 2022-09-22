import { computeNodeAndOffset, computeSelectionOffset, countCodeunit } from './dom';

let editor: HTMLDivElement;

beforeAll(() => {
    editor = document.createElement('div');
    editor.setAttribute('contentEditable', 'true');
});

describe('computeNodeAndOffset', () => {
    test('Should find at the start of simple text', () => {
        // When
        editor.innerHTML = 'abcdefgh';
        const { node, offset } = computeNodeAndOffset(editor, 0);

        // Then
        expect(node).toBe(editor.childNodes[0]);
        expect(offset).toBe(0);
    });

    test('Should find in the middle of simple text', () => {
        // When
        editor.innerHTML = 'abcdefgh';
        const { node, offset } = computeNodeAndOffset(editor, 4);

        // Then
        expect(node).toBe(editor.childNodes[0]);
        expect(offset).toBe(4);
    });

    test('Should find at the end of simple text', () => {
        // When
        editor.innerHTML = 'abcdefgh';
        const { node, offset } = computeNodeAndOffset(editor, 8);

        // Then
        expect(node).toBe(editor.childNodes[0]);
        expect(offset).toBe(8);
    });

    test('Should returns null if off the end', () => {
        // When
        editor.innerHTML = 'abcdefgh';
        const { node, offset } = computeNodeAndOffset(editor, 9);

        // Then
        expect(node).toBeNull();
        expect(offset).toBe(1);
    });

    test('Should find before subnode', () => {
        // When
        editor.innerHTML = 'abc<b>def</b>gh';
        const { node, offset } = computeNodeAndOffset(editor, 2);

        // Then
        expect(node).toBe(editor.childNodes[0]);
        expect(offset).toBe(2);
    });

    test('Should find after subnode', () => {
        // When
        editor.innerHTML = 'abc<b>def</b>gh';
        const { node, offset } = computeNodeAndOffset(editor, 4);

        // Then
        expect(node).toBe(editor.childNodes[1].childNodes[0]);
        expect(offset).toBe(1);
    });

    test('Should find inside subnode', () => {
        // When
        editor.innerHTML = 'abc<b>def</b>gh';
        const { node, offset } = computeNodeAndOffset(editor, 7);

        // Then
        expect(node).toBe(editor.childNodes[2]);
        expect(offset).toBe(1);
    });

    test('Should find after subnode', () => {
        // When
        editor.innerHTML = 'abc<b>def</b>gh';
        const { node, offset } = computeNodeAndOffset(editor, 7);

        // Then
        expect(node).toBe(editor.childNodes[2]);
        expect(offset).toBe(1);
    });

    test('Should find before br', () => {
        // When
        editor.innerHTML = 'a<br />b';
        const { node, offset } = computeNodeAndOffset(editor, 0);

        // Then
        expect(node).toBe(editor.childNodes[0]);
        expect(offset).toBe(0);
    });

    test('Should find br start', () => {
        // When
        editor.innerHTML = 'a<br />b';
        const { node, offset } = computeNodeAndOffset(editor, 1);

        // Then
        expect(node).toBe(editor.childNodes[0]);
        expect(offset).toBe(1);
    });

    test('Should find br end', () => {
        // When
        editor.innerHTML = 'a<br />b';
        const { node, offset } = computeNodeAndOffset(editor, 2);

        // Then
        expect(node).toBe(editor.childNodes[2]);
        expect(offset).toBe(0);
    });

    test('Should find between br', () => {
        // When
        editor.innerHTML = 'a<br /><br />b';
        const { node, offset } = computeNodeAndOffset(editor, 2);

        // Then
        expect(node).toBe(editor.childNodes[2]);
        expect(offset).toBe(0);
    });

    test('Should find after br', () => {
        // When
        editor.innerHTML = 'a<br />b';
        const { node, offset } = computeNodeAndOffset(editor, 3);

        // Then
        expect(node).toBe(editor.childNodes[2]);
        expect(offset).toBe(1);
    });

    test('Should find inside an empty list', () => {
        // When
        editor.innerHTML = '<ul><li><li></ul>';
        const { node, offset } = computeNodeAndOffset(editor, 0);

        // Then
        expect(node).toBe(editor.childNodes[0].childNodes[0]);
        expect(offset).toBe(0);
    });

    test('Should find inside two empty list', () => {
        // When
        editor.innerHTML = '<ul><li><li></ul><li><li></ul>';
        const { node, offset } = computeNodeAndOffset(editor, 0);

        // Then
        expect(node).toBe(editor.childNodes[0].childNodes[0]);
        expect(offset).toBe(0);
    });

    test('Should find inside a list', () => {
        // When
        editor.innerHTML = '<ul><li>foo<li></ul>';
        const { node, offset } = computeNodeAndOffset(editor, 1);

        // Then
        expect(node).toBe(editor.childNodes[0].childNodes[0].childNodes[0]);
        expect(offset).toBe(1);
    });
});

describe('computeSelectionOffset', () => {
    test('Should contain all the characters when the editor node is selected', () => {
        // When
        editor.innerHTML = 'abc<b>def</b>gh';
        // Use the editor node and a offset as 1 to simulate the FF behavior
        let offset = computeSelectionOffset(editor, 1);

        // Then
        expect(offset).toBe(8);

        // When
        editor.innerHTML = 'abc<b>def</b>gh<ul><li>alice</li><li>bob</li>';
        offset = computeSelectionOffset(editor, 1);

        // Then
        expect(offset).toBe(16);
    });

    test('Should contains the selected characters', () => {
        // When
        editor.innerHTML = 'abc<b>def</b>gh<ul><li>alice</li><li>bob</li>';
        let offset = computeSelectionOffset(editor.childNodes[0], 1);

        // Then
        expect(offset).toBe(1);

        // When
        offset = computeSelectionOffset(editor.childNodes[0], 20);

        // Then
        expect(offset).toBe(20);
    });
});

describe('countCodeunit', () => {
    test('Should count ASCII', () => {
        // When
        editor.innerHTML = 'abcdefgh';
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

    test('Should count UCS-2', () => {
        // When
        editor.innerHTML = 'a\u{03A9}b\u{03A9}c';
        const textNode = editor.childNodes[0];

        // Then
        expect(countCodeunit(editor, textNode, 0)).toBe(0);
        expect(countCodeunit(editor, textNode, 1)).toBe(1);
        expect(countCodeunit(editor, textNode, 4)).toBe(4);
        expect(countCodeunit(editor, textNode, 5)).toBe(5);
        expect(countCodeunit(editor, textNode, 6)).toBe(-1);
    });

    test('Should count complex', () => {
        // When
        editor.innerHTML = 'a\u{1F469}\u{1F3FF}\u{200D}\u{1F680}b';
        const textNode = editor.childNodes[0];

        // Then
        expect(countCodeunit(editor, textNode, 0)).toBe(0);
        expect(countCodeunit(editor, textNode, 7)).toBe(7);
        expect(countCodeunit(editor, textNode, 8)).toBe(8);
        expect(countCodeunit(editor, textNode, 9)).toBe(9);
        expect(countCodeunit(editor, textNode, 10)).toBe(-1);
    });

    test('Should count nested', () => {
        // When
        editor.innerHTML = 'a<b>b</b>c';
        const firstTextNode = editor.childNodes[0];
        const boldTextNode = editor.childNodes[1].childNodes[0];
        const thirdTextNode = editor.childNodes[2];

        // Then
        expect(countCodeunit(editor, firstTextNode, 0)).toBe(0);
        expect(countCodeunit(editor, boldTextNode, 0)).toBe(1);
        expect(countCodeunit(editor, thirdTextNode, 0)).toBe(2);
    });

    test('Should treats br as a character', () => {
        // When
        editor.innerHTML = 'a<br />b';
        const firstTextNode = editor.childNodes[0];
        const brNode = editor.childNodes[1];
        const secondTextNode = editor.childNodes[2];

        // Then
        expect(countCodeunit(editor, firstTextNode, 0)).toBe(0);
        expect(countCodeunit(editor, brNode, 0)).toBe(1);
        expect(countCodeunit(editor, brNode, 1)).toBe(2);
        expect(countCodeunit(editor, secondTextNode, 1)).toBe(3);
    });

    test('Should works with deeply nested', () => {
        // When
        editor.innerHTML = 'aaa<b><i>bbb</i>ccc</b>ddd';
        const firstTextNode = editor.childNodes[0];
        const boldItalicNode = editor.childNodes[1].childNodes[0];
        const boldItalicTextNode = editor.childNodes[1].childNodes[0].childNodes[0];
        const boldOnlyNode = editor.childNodes[1].childNodes[1];
        const thirdTextNode = editor.childNodes[2];

        // Then
        expect(countCodeunit(editor, firstTextNode, 1)).toBe(1);
        expect(countCodeunit(editor, firstTextNode, 2)).toBe(2);
        expect(countCodeunit(editor, firstTextNode, 3)).toBe(3);
        expect(countCodeunit(editor, boldItalicNode, 0)).toBe(3);
        expect(countCodeunit(editor, boldItalicNode, 1)).toBe(4);
        expect(countCodeunit(editor, boldItalicNode, 2)).toBe(5);
        // We can supply the text node or its parent
        expect(countCodeunit(editor, boldItalicTextNode, 0)).toBe(3);
        expect(countCodeunit(editor, boldItalicTextNode, 1)).toBe(4);
        expect(countCodeunit(editor, boldItalicTextNode, 2)).toBe(5);
        expect(countCodeunit(editor, boldOnlyNode, 0)).toBe(6);
        expect(countCodeunit(editor, boldOnlyNode, 1)).toBe(7);
        expect(countCodeunit(editor, boldOnlyNode, 2)).toBe(8);
        expect(countCodeunit(editor, thirdTextNode, 0)).toBe(9);
        expect(countCodeunit(editor, thirdTextNode, 1)).toBe(10);
        expect(countCodeunit(editor, thirdTextNode, 2)).toBe(11);
    });
});
