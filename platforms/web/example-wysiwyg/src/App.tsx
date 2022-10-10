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

import { useWysiwyg } from '@matrix-org/matrix-wysiwyg';

import './App.css';

function App() {
    const { ref, isWysiwygReady, wysiwyg } = useWysiwyg({
        isAutoFocusEnabled: true,
    });

    return (
        <div>
            <button onClick={wysiwyg.undo}>undo</button>
            <button onClick={wysiwyg.redo}>redo</button>
            <button onClick={wysiwyg.bold}>bold</button>
            <button onClick={wysiwyg.italic}>italic</button>
            <button onClick={wysiwyg.underline}>underline</button>
            <button onClick={wysiwyg.strikeThrough}>strikeThrough</button>
            <button onClick={wysiwyg.orderedList}>orderedList</button>
            <button onClick={wysiwyg.unorderedList}>unorderedList</button>
            <button onClick={wysiwyg.inlineCode}>inlineCode</button>
            <button onClick={wysiwyg.clear}>clear</button>
            <div ref={ref} contentEditable={isWysiwygReady} />
        </div>
    );
}

export default App;
