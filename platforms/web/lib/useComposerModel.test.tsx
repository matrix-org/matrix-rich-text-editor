/*
Copyright 2023 The Matrix.org Foundation C.I.C.

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

import { act, RefObject } from 'react';
import { renderHook, waitFor } from '@testing-library/react';

import * as mockRustModel from '../generated/wysiwyg';
import { useComposerModel } from './useComposerModel';

describe('useComposerModel', () => {
    let mockComposer: HTMLDivElement;
    let mockNullRef: RefObject<null>;
    let mockComposerRef: RefObject<HTMLElement>;

    beforeEach(() => {
        mockComposer = document.createElement('div');
        mockNullRef = { current: null };
        mockComposerRef = {
            current: mockComposer,
        };
        vi.spyOn(mockRustModel, 'new_composer_model');
        vi.spyOn(mockRustModel, 'new_composer_model_from_html');
    });

    afterEach(() => {
        vi.clearAllMocks();
    });

    afterAll(() => {
        vi.restoreAllMocks();
    });

    it('Does not create a composerModel without a ref', () => {
        const { result } = renderHook(() => useComposerModel(mockNullRef));

        expect(result.current.composerModel).toBeNull();
    });

    it('Only calls `new_composer_model` if ref exists but no initial content exists', async () => {
        const { result } = renderHook(() => useComposerModel(mockComposerRef));

        // wait for the composerModel to be created
        await waitFor(() => {
            expect(result.current.composerModel).not.toBeNull();
        });

        // check only new_composer_model has been called
        expect(mockRustModel.new_composer_model).toHaveBeenCalledTimes(1);
        expect(
            mockRustModel.new_composer_model_from_html,
        ).not.toHaveBeenCalled();
    });

    it('Calls `new_composer_model_from_html` if ref and initial content exists', async () => {
        const { result } = renderHook(() =>
            useComposerModel(mockComposerRef, 'some content'),
        );

        // wait for the composerModel to be created
        await waitFor(() => {
            expect(result.current.composerModel).not.toBeNull();
        });

        // check only new_composer_model_from_html has been called
        expect(
            mockRustModel.new_composer_model_from_html,
        ).toHaveBeenCalledTimes(1);
        expect(mockRustModel.new_composer_model).not.toHaveBeenCalled();
    });

    it('Sets the ref inner html when initial content is valid html', async () => {
        const inputContent = `<a href="this is allowed" other="disallowedattribute">test link</a>`;

        // the rust model will strip "bad" attributes and the hook always adds a trailing <br>
        const expectedComposerInnerHtml = `<a href="this is allowed">test link</a><br>`;
        const { result } = renderHook(() =>
            useComposerModel(mockComposerRef, inputContent),
        );

        // wait for the composerModel to be created
        await waitFor(() => {
            expect(result.current.composerModel).not.toBeNull();
        });

        // check that the content of the div is the rust model output
        expect(mockComposer.innerHTML).toBe(expectedComposerInnerHtml);
    });

    it('Falls back to calling `new_composer_model` if there is a parsing error', async () => {
        // Use badly formed initial content to cause a html parsing error
        const { result } = renderHook(() =>
            useComposerModel(mockComposerRef, '<badly>formed content</>'),
        );

        // wait for the composerModel to be created
        await waitFor(() => {
            expect(result.current.composerModel).not.toBeNull();
        });

        // check that both functions have been called
        expect(
            mockRustModel.new_composer_model_from_html,
        ).toHaveBeenCalledTimes(1);
        expect(mockRustModel.new_composer_model).toHaveBeenCalledTimes(1);
    });

    it("Doesn't double intialize the model if customSuggestionPatterns are set", async () => {
        let useProps: {
            editorRef: RefObject<HTMLElement | null>;
            initialContent?: string;
            customSuggestionPatterns?: Array<string>;
        } = {
            editorRef: mockComposerRef,
            initialContent: '',
            customSuggestionPatterns: undefined,
        };

        const { result, rerender } = renderHook(
            (props: {
                editorRef: RefObject<HTMLElement | null>;
                initialContent?: string;
                customSuggestionPatterns?: Array<string>;
            }) => {
                let a = useComposerModel(
                    props.editorRef,
                    props.initialContent,
                    props.customSuggestionPatterns,
                );
                return a;
            },
            { initialProps: useProps },
        );

        // wait for the composerModel to be created
        await waitFor(() => {
            expect(result.current.composerModel).not.toBeNull();
        });

        await act(() => {
            useProps.customSuggestionPatterns = ['test'];
            rerender(useProps);
        });

        expect(mockRustModel.new_composer_model).toHaveBeenCalledTimes(1);
        expect(
            mockRustModel.new_composer_model_from_html,
        ).not.toHaveBeenCalled();
    });
});
