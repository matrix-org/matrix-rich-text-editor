import { computeNodeAndOffset, computeSelectionOffset } from './dom';

describe('computeNodeAndOffset', () => {
    let editor: HTMLDivElement;

    beforeAll(() => {
        editor = document.createElement('div');
        editor.setAttribute('contentEditable', 'true');
    });

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
    let editor: HTMLDivElement;

    beforeAll(() => {
        editor = document.createElement('div');
        editor.setAttribute('contentEditable', 'true');
    });

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
