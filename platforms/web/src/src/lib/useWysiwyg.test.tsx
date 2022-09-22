import { render } from '@testing-library/react';
import { forwardRef } from 'react';

import { useWysiwyg } from './useWysiwyg';

const Editor = forwardRef<HTMLDivElement>(function Editor(props, forwardRef) {
    const { ref, isWysiwygReady } = useWysiwyg();
    return <div ref={(node) => {
        if (node) {
            ref.current = node;
            if (typeof forwardRef === 'function') forwardRef(node);
            else if (forwardRef) forwardRef.current = node;
        }
    }}
    contentEditable={isWysiwygReady} />;
});

describe('useWysiwyg', () => {
    describe('Rendering characters', () => {
        let editor: HTMLDivElement;

        function deleteRange(start: number, end: number) {
            const textNode = editor.childNodes[0];
            const range = document.createRange();
            range.setStart(textNode, start);
            range.setEnd(textNode, end);
            const sel = document.getSelection();
            if (sel) {
                sel.removeAllRanges();
                sel.addRange(range);
                sel.deleteFromDocument();
            }
        }

        beforeAll(() => {
            render(<Editor ref={(node) => {
                if (node) {
                    editor = node;
                }
            }} />);
        });

        test('Should render ASCII characters with width 1', () => {
            // When
            editor.innerHTML = 'abcd';
            deleteRange(0, 1);

            // Then
            expect(editor.innerHTML).toBe('bcd');

            //When
            editor.innerHTML = 'abcd';
            deleteRange(0, 2);

            //Then
            expect(editor.innerHTML).toBe('cd');
        });

        test.skip('Should render UCS-2 characters with width 1', () => {
            // When
            editor.innerHTML = '\u{03A9}bcd';
            deleteRange(0, 1);

            // Then
            expect(editor.innerHTML).toBe('bcd');

            // When
            editor.innerHTML = '\u{03A9}bcd';
            deleteRange(0, 2);

            // Then
            expect(editor.innerHTML).toBe('cd');
        });

        test.skip('Should render Multi-code unit UTF-16 characters with width 2', () => {
            // When
            editor.innerHTML = '\u{1F4A9}bcd';
            deleteRange(0, 2);

            // Then
            expect(editor.innerHTML).toBe('bcd');

            //When
            editor.innerHTML = '\u{1F4A9}bcd';
            deleteRange(0, 3);

            //Then
            expect(editor.innerHTML).toBe('cd');
        });

        test.skip('Should render complex characters with width = num UTF-16 code units', () => {
            // When
            editor.innerHTML = '\u{1F469}\u{1F3FF}\u{200D}\u{1F680}bcd';
            deleteRange(0, 7);

            // Then
            expect(editor.innerHTML).toBe('bcd');

            //When
            editor.innerHTML = '\u{1F469}\u{1F3FF}\u{200D}\u{1F680}bcd';
            deleteRange(0, 8);

            //Then
            expect(editor.innerHTML).toBe('cd');
        });
    });
});
