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

import { render, screen, waitFor } from '@testing-library/react';

import { Editor } from './testUtils/Editor';
import { deleteRange } from './testUtils/selection';

describe('useWysiwyg', () => {
    describe('Rendering characters', () => {
        let editor: HTMLDivElement;

        function setEditorHtml(html: string) {
            // The editor always needs an extra BR after your HTML
            editor.innerHTML = html + '<br />';
        }

        beforeAll(async () => {
            render(
                <Editor
                    ref={(node) => {
                        if (node) {
                            editor = node;
                        }
                    }}
                />,
            );
            await waitFor(() =>
                expect(screen.getByRole('textbox')).toHaveAttribute(
                    'contenteditable',
                    'true',
                ),
            );
        });

        it.only('Should render ASCII characters with width 1', () => {
            // When
            setEditorHtml('abcd');
            screen.debug();
            deleteRange(editor, 0, 1);
            // Then
            expect(editor).toContainHTML('bcd');
            //When
            setEditorHtml('abcd');
            deleteRange(editor, 0, 2);
            //Then
            expect(editor).toContainHTML('cd');
        });

        it('Should render UCS-2 characters with width 1', () => {
            // When
            setEditorHtml('\u{03A9}bcd');
            deleteRange(editor, 0, 1);

            // Then
            expect(editor).toContainHTML('bcd');

            // When
            setEditorHtml('\u{03A9}bcd');
            deleteRange(editor, 0, 2);

            // Then
            expect(editor).toContainHTML('cd');
        });

        it('Should render Multi-code unit UTF-16 chars width 2', () => {
            // When
            setEditorHtml('\u{1F4A9}bcd');
            deleteRange(editor, 0, 2);

            // Then
            expect(editor).toContainHTML('bcd');

            //When
            setEditorHtml('\u{1F4A9}bcd');
            deleteRange(editor, 0, 3);

            //Then
            expect(editor).toContainHTML('cd');
        });

        it('Should render complex chars width = UTF-16 code units', () => {
            // When
            setEditorHtml('\u{1F469}\u{1F3FF}\u{200D}\u{1F680}bcd');
            deleteRange(editor, 0, 7);

            // Then
            expect(editor).toContainHTML('bcd');

            //When
            setEditorHtml('\u{1F469}\u{1F3FF}\u{200D}\u{1F680}bcd');
            deleteRange(editor, 0, 8);

            //Then
            expect(editor).toContainHTML('cd');
        });
    });

    test('Create wysiwyg with initial content', async () => {
        // when
        const content = 'fo<strong>o</strong><br />b<em>ar</em>';
        render(<Editor initialContent={content} />);

        // Then
        await waitFor(() =>
            expect(screen.getByRole('textbox')).toContainHTML(content),
        );
    });

    test('Handle panic', async () => {
        // When
        const content = 'fo<strng>o</strng><br />b<em><kar</em>';
        render(<Editor initialContent={content} />);

        // Then
        await waitFor(() =>
            expect(screen.getByRole('textbox')).not.toContainHTML(content),
        );
    });
});
