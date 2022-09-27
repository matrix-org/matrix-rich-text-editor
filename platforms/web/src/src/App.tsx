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

import { MouseEventHandler } from 'react';

import { useWysiwyg } from '../lib/useWysiwyg';
import boldImage from './images/bold.svg';
import undoImage from './images/undo.svg';
import redoImage from './images/redo.svg';
import italicImage from './images/italic.svg';
import underlineImage from './images/underline.svg';
import strikeTroughImage from './images/strike_through.svg';
import listUnorderedImage from './images/list-unordered.svg';
import listOrderedImage from './images/list-ordered.svg';

type ButtonProps = {
  onClick: MouseEventHandler<HTMLButtonElement>;
  imagePath: string;
  alt: string;
};

function Button({ onClick, imagePath, alt }: ButtonProps) {
    return (
        <button type="button" onClick={onClick}>
            <img alt={alt} src={imagePath} />
        </button>);
}

function App() {
    const { ref, isWysiwygReady, wysiwyg, debug } = useWysiwyg({ isAutoFocusEnabled: true });

    return (
        <div className="wrapper">
            <div>
                <div className="editor_container">
                    <div className="editor_toolbar">
                        <Button onClick={wysiwyg.undo} alt="undo" imagePath={undoImage} />
                        <Button onClick={wysiwyg.redo} alt="redo" imagePath={redoImage} />
                        <Button onClick={wysiwyg.bold} alt="bold" imagePath={boldImage} />
                        <Button onClick={wysiwyg.italic} alt="italic" imagePath={italicImage} />
                        <Button onClick={wysiwyg.underline} alt="underline" imagePath={underlineImage} />
                        <Button onClick={wysiwyg.strikeThrough} alt="strike through" imagePath={strikeTroughImage} />
                        <Button onClick={wysiwyg.unorderedList} alt="list unordered" imagePath={listUnorderedImage} />
                        <Button onClick={wysiwyg.orderedList} alt="list ordered" imagePath={listOrderedImage} />
                        <Button onClick={wysiwyg.inlineCode} alt="inline code" imagePath={listOrderedImage} />
                    </div>
                    <div className="editor" ref={ref} contentEditable={isWysiwygReady} />
                </div>
            </div>
            <h2>Model:</h2>
            <div className="dom" ref={debug.modelRef} />
            <h2>Test case: <button type="button" onClick={debug.resetTestCase}>Start from here</button></h2>
            <div className="testCase" ref={debug.testRef}>
                let mut model = cm("");<br />
                assert_eq!(tx(&amp;model), "");
            </div>
        </div>);
}

export default App;
