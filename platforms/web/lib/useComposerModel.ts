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

import { RefObject, useCallback, useEffect, useState } from 'react';

// rust generated bindings
import init, {
    ComposerModel,
    // eslint-disable-next-line camelcase
    new_composer_model,
    // eslint-disable-next-line camelcase
    new_composer_model_from_html,
} from '../generated/wysiwyg.js';
import { replaceEditor } from './dom';

let initStarted = false;
let initFinished = false;

/**
 * Initialise the WASM module, or do nothing if it is already initialised.
 */
export async function initOnce(): Promise<void> {
    if (initFinished) {
        return Promise.resolve();
    }
    if (initStarted) {
        // Wait until the other init call has finished
        return new Promise<void>((resolve) => {
            function tryResolve(): void {
                if (initFinished) {
                    resolve();
                }
                setTimeout(tryResolve, 200);
            }
            tryResolve();
        });
    }

    initStarted = true;
    await init();
    initFinished = true;
}

export function useComposerModel(
    editorRef: RefObject<HTMLElement | null>,
    initialContent?: string,
    customSuggestionPatterns?: Array<string>,
): {
    composerModel: ComposerModel | null;
    onError: (initialContent?: string) => Promise<void>;
} {
    const [composerModel, setComposerModel] = useState<ComposerModel | null>(
        null,
    );

    const initModel = useCallback(
        async (initialContent?: string) => {
            await initOnce();

            let contentModel: ComposerModel;
            if (initialContent) {
                try {
                    const newModel = new_composer_model_from_html(
                        initialContent,
                        0,
                        initialContent.length,
                    );
                    contentModel = newModel;

                    if (editorRef.current) {
                        // we need to use the rust model as the source of truth, to allow it to do things
                        // like add attributes to mentions automatically
                        const modelContent = newModel.get_content_as_html();
                        replaceEditor(
                            editorRef.current,
                            modelContent,
                            0,
                            modelContent.length,
                        );
                    }
                } catch (e) {
                    // if the initialisation fails, due to a parsing failure of the html, fallback to an empty composer
                    contentModel = new_composer_model();
                }
            } else {
                contentModel = new_composer_model();
            }
            setComposerModel(contentModel);
        },
        [setComposerModel, editorRef],
    );

    useEffect(() => {
        if (composerModel && customSuggestionPatterns) {
            composerModel.set_custom_suggestion_patterns(
                customSuggestionPatterns,
            );
        }
    }, [composerModel, customSuggestionPatterns]);

    useEffect(() => {
        if (editorRef.current) {
            initModel(initialContent);
        }
    }, [editorRef, initModel, initialContent]);

    // If a panic occurs, we call onError to attempt to reinitialise the composer
    // with some plain text (called in useListeners).
    return { composerModel, onError: initModel };
}
