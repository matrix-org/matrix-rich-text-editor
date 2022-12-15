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

import { RefObject, useEffect, useState } from 'react';

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
export async function initOnce() {
    if (initFinished) {
        return Promise.resolve();
    }
    if (initStarted) {
        // Wait until the other init call has finished
        return new Promise<void>((resolve) => {
            function tryResolve() {
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
) {
    const [composerModel, setComposerModel] = useState<ComposerModel | null>(
        null,
    );

    useEffect(() => {
        const initModel = async () => {
            await initOnce();

            if (initialContent) {
                setComposerModel(
                    new_composer_model_from_html(
                        initialContent,
                        0,
                        initialContent.length,
                    ),
                );

                if (editorRef.current) {
                    replaceEditor(
                        editorRef.current,
                        initialContent,
                        0,
                        initialContent.length,
                    );
                }
            } else {
                setComposerModel(new_composer_model());
            }
        };

        if (editorRef.current) {
            initModel();
        }
    }, [editorRef, initialContent]);

    return composerModel;
}
