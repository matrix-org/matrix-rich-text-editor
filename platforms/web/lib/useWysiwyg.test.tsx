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

import { fireEvent, render, screen, waitFor } from '@testing-library/react';

import { Editor } from './testUtils/Editor';
import { deleteRange } from './testUtils/selection';

describe('useWysiwyg', () => {
    describe('Rendering characters', () => {
        let editor: HTMLDivElement;

        function setEditorHtml(html: string): void {
            // The editor always needs an extra BR after your HTML
            editor.innerHTML = html + '<br />';
        }

        beforeEach(async () => {
            render(
                <Editor
                    ref={(node): void => {
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

        it('Should render ASCII characters with width 1', () => {
            // When
            setEditorHtml('abcd');
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

        it('Should render characters based on composition events', () => {
            // simulate the event when entering `¨` (option+u on mac)
            const startCompositionEvent = new CompositionEvent(
                'compositionstart',
            );

            // simulate the event when then pressing `u`
            const compositionData = 'ü';
            const endCompositionEvent = new CompositionEvent('compositionend', {
                data: compositionData,
            });

            fireEvent(editor, startCompositionEvent);
            fireEvent(editor, endCompositionEvent);

            expect(editor).toHaveTextContent(compositionData);
        });
    });
    test('Initialising composer with a mention displays all mention attributes', async () => {
        const testUser = 'TEST_USER';
        const testStyle = 'MOCK;STYLE';
        const content = `<a href="https://matrix.to/#/@test_user:element.io" style=${testStyle}>${testUser}</a> `;
        render(<Editor initialContent={content} />);

        // Wait for the mention to appear on the screen, then check it has the attributes
        // required for correct display in the composer.
        const mention = await screen.findByText(testUser);

        // these attributes are automatically added by the rust model
        expect(mention).toHaveAttribute('data-mention-type', 'user');
        expect(mention).toHaveAttribute('contenteditable', 'false');

        // this attribute is passed through, from the html into the rust model
        expect(mention).toHaveAttribute('style', testStyle);
    });

    test('Create wysiwyg with initial content', async () => {
        // Given
        const content = 'fo<strong>o</strong><br />b<em>ar</em>';
        const processedContent =
            '<p>fo<strong>o</strong></p><p>b<em>ar</em></p>';

        // When
        render(<Editor initialContent={content} />);

        // Then
        await waitFor(() =>
            expect(screen.getByRole('textbox')).toContainHTML(processedContent),
        );
    });

    test('Handle panic from invalid html in initial content', async () => {
        // When we have an invalid tag
        const invalidHtml = 'fo<strng>o</strng><br />b<em><kar</em>';

        // Then
        expect(() =>
            render(<Editor initialContent={invalidHtml} />),
        ).not.toThrow();
    });
});
