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

import { forwardRef } from 'react';

import { useWysiwyg } from '../useWysiwyg';

interface EditorProps {
    initialContent?: string;
}

export const Editor = forwardRef<HTMLDivElement, EditorProps>(function Editor(
    { initialContent }: EditorProps,
    forwardRef,
) {
    const { ref, isWysiwygReady, wysiwyg, actionStates, content } = useWysiwyg({
        initialContent,
    });

    const keys = Object.keys(wysiwyg) as Array<keyof typeof wysiwyg>;
    return (
        <>
            {keys.map((key) => (
                <button
                    key={key}
                    type="button"
                    onClick={() => wysiwyg[key]()}
                    data-state={actionStates[key]}
                >
                    {key}
                </button>
            ))}
            <div
                ref={(node) => {
                    if (node) {
                        ref.current = node;
                        if (typeof forwardRef === 'function') forwardRef(node);
                        else if (forwardRef) forwardRef.current = node;
                    }
                }}
                contentEditable={isWysiwygReady}
                role="textbox"
                data-content={content}
            />
        </>
    );
});
