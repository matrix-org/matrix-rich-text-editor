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

import { MouseEventHandler, useState } from 'react';

import { useWysiwyg } from '../lib/useWysiwyg';
import boldImage from './images/bold.svg';
import undoImage from './images/undo.svg';
import redoImage from './images/redo.svg';
import italicImage from './images/italic.svg';
import underlineImage from './images/underline.svg';
import strikeTroughImage from './images/strike_through.svg';
import listUnorderedImage from './images/list_unordered.svg';
import listOrderedImage from './images/list_ordered.svg';
import inlineCodeImage from './images/inline_code.svg';
import codeBlockImage from './images/code_block.svg';
import quoteImage from './images/quote.svg';
import indentImage from './images/indent.svg';
import unindentImage from './images/unindent.svg';
import { Wysiwyg, WysiwygEvent } from '../lib/types';

type ButtonProps = {
    onClick: MouseEventHandler<HTMLButtonElement>;
    imagePath: string;
    alt: string;
    state: 'enabled' | 'disabled' | 'reversed';
};

function Button({ onClick, imagePath, alt, state }: ButtonProps) {
    const isReversed = state === 'reversed';
    const isDisabled = state === 'disabled';
    return (
        <button
            type="button"
            onClick={onClick}
            style={{
                ...(isReversed && { backgroundColor: 'lightgray' }),
                ...(isDisabled && { backgroundColor: 'firebrick' }),
            }}
        >
            <img alt={alt} src={imagePath} />
        </button>
    );
}

function App() {
    const [enterToSend, setEnterToSend] = useState(true);

    const inputEventProcessor = (
        e: WysiwygEvent,
        wysiwyg: Wysiwyg,
    ): WysiwygEvent | null => {
        if (e instanceof ClipboardEvent) {
            return e;
        }

        if (
            !(e instanceof KeyboardEvent) &&
            ((enterToSend && e.inputType === 'insertParagraph') ||
                e.inputType === 'sendMessage')
        ) {
            if (debug.testRef.current) {
                debug.traceAction(null, 'send', `${wysiwyg.content()}`);
            }
            console.log(`SENDING: ${wysiwyg.content()}`);
            wysiwyg.actions.clear();
            return null;
        }

        return e;
    };

    const { ref, isWysiwygReady, actionStates, wysiwyg, debug, suggestion } =
        useWysiwyg({
            isAutoFocusEnabled: true,
            inputEventProcessor,
        });

    const onEnterToSendChanged = () => {
        setEnterToSend((prevValue) => !prevValue);
    };

    const isInList =
        actionStates.unorderedList === 'reversed' ||
        actionStates.orderedList === 'reversed';

    return (
        <div className="wrapper">
            <div>
                <div className="editor_container">
                    <div className="editor_toolbar">
                        <Button
                            onClick={wysiwyg.undo}
                            alt="undo"
                            imagePath={undoImage}
                            state={actionStates.undo}
                        />
                        <Button
                            onClick={wysiwyg.redo}
                            alt="redo"
                            imagePath={redoImage}
                            state={actionStates.redo}
                        />
                        <Button
                            onClick={wysiwyg.bold}
                            alt="bold"
                            imagePath={boldImage}
                            state={actionStates.bold}
                        />
                        <Button
                            onClick={wysiwyg.italic}
                            alt="italic"
                            imagePath={italicImage}
                            state={actionStates.italic}
                        />
                        <Button
                            onClick={wysiwyg.underline}
                            alt="underline"
                            imagePath={underlineImage}
                            state={actionStates.underline}
                        />
                        <Button
                            onClick={wysiwyg.strikeThrough}
                            alt="strike through"
                            imagePath={strikeTroughImage}
                            state={actionStates.strikeThrough}
                        />
                        <Button
                            onClick={wysiwyg.unorderedList}
                            alt="list unordered"
                            imagePath={listUnorderedImage}
                            state={actionStates.unorderedList}
                        />
                        <Button
                            onClick={wysiwyg.orderedList}
                            alt="list ordered"
                            imagePath={listOrderedImage}
                            state={actionStates.orderedList}
                        />
                        {isInList && (
                            <Button
                                onClick={wysiwyg.indent}
                                alt="indent"
                                imagePath={indentImage}
                                state={actionStates.indent}
                            />
                        )}
                        {isInList && (
                            <Button
                                onClick={wysiwyg.unindent}
                                alt="unindent"
                                imagePath={unindentImage}
                                state={actionStates.unindent}
                            />
                        )}
                        <Button
                            onClick={wysiwyg.quote}
                            alt="quote"
                            imagePath={quoteImage}
                            state={actionStates.quote}
                        />
                        <Button
                            onClick={wysiwyg.inlineCode}
                            alt="inline code"
                            imagePath={inlineCodeImage}
                            state={actionStates.inlineCode}
                        />
                        <Button
                            onClick={wysiwyg.codeBlock}
                            alt="code block"
                            imagePath={codeBlockImage}
                            state={actionStates.codeBlock}
                        />
                        <button type="button" onClick={(_e) => wysiwyg.clear()}>
                            clear
                        </button>
                        {suggestion && suggestion.type === 'mention' && (
                            <button
                                type="button"
                                onClick={(_e) =>
                                    wysiwyg.mention(
                                        'https://matrix.to/#/@alice_user:element.io',
                                        'Alice',
                                        {
                                            'contentEditable': 'false',
                                            'data-mention-type':
                                                suggestion.keyChar === '@'
                                                    ? 'user'
                                                    : 'room',
                                        },
                                    )
                                }
                            >
                                Add {suggestion.keyChar}mention
                            </button>
                        )}
                        {suggestion && suggestion.type === 'command' && (
                            <button
                                type="button"
                                onClick={(_e) => wysiwyg.command('/spoiler ')}
                            >
                                Add /spoiler command
                            </button>
                        )}
                    </div>
                    <div
                        className="editor"
                        ref={ref}
                        contentEditable={isWysiwygReady}
                    />
                </div>
                <div className="editor_options">
                    <input
                        type="checkbox"
                        id="enterToSend"
                        checked={enterToSend}
                        onChange={onEnterToSendChanged}
                    />
                    <label htmlFor="enterToSend">
                        Enter to "send" (if unchecked, use Ctrl+Enter)
                    </label>
                </div>
            </div>
            <h2>Model:</h2>
            <div className="dom" ref={debug.modelRef} />
            <h2>
                Test case:{' '}
                <button type="button" onClick={debug.resetTestCase}>
                    Start from here
                </button>
            </h2>
            <div className="testCase" ref={debug.testRef}>
                let mut model = cm("");
                <br />
                assert_eq!(tx(&amp;model), "");
            </div>
        </div>
    );
}

export default App;
