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

import { forwardRef, MutableRefObject } from 'react';

import { FormattingFunctions, InputEventProcessor } from '../types';
import { useWysiwyg } from '../useWysiwyg';

interface EditorProps {
    initialContent?: string;
    inputEventProcessor?: InputEventProcessor;
    actionsRef?: MutableRefObject<FormattingFunctions | null>;
}

export const Editor = forwardRef<HTMLDivElement, EditorProps>(function Editor(
    { initialContent, inputEventProcessor, actionsRef }: EditorProps,
    forwardRef,
) {
    const { ref, isWysiwygReady, wysiwyg, actionStates, content } = useWysiwyg({
        initialContent,
        inputEventProcessor,
    });

    if (actionsRef) actionsRef.current = wysiwyg;

    const keys = Object.keys(wysiwyg).filter(
        (key) =>
            key !== 'insertText' &&
            key !== 'link' &&
            key !== 'removeLinks' &&
            key !== 'getLink',
    ) as Array<
        Exclude<
            keyof typeof wysiwyg,
            'insertText' | 'link' | 'removeLinks' | 'getLink'
        >
    >;
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
            <button
                type="button"
                onClick={() => wysiwyg.insertText('add new words')}
            >
                insertText
            </button>
            <button type="button" onClick={() => wysiwyg.link('my link')}>
                link
            </button>
            <button
                type="button"
                onClick={() => wysiwyg.link('my link', 'my text')}
            >
                link with text
            </button>
            <button type="button" onClick={() => wysiwyg.removeLinks()}>
                remove links
            </button>
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
