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

// We are using node api here
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-nocheck

import '@testing-library/jest-dom';
import { cleanup } from '@testing-library/react';
import fs from 'node:fs/promises';
import path from 'path';

globalThis.fetch = (url) => {
    // wysiwyg.js binding uses fetch to get the wasm file
    // we return manually here the wasm file
    if (url instanceof URL && url.href.includes('wysiwyg_bg.wasm')) {
        const wasmPath = path.resolve(__dirname, 'generated/wysiwyg_bg.wasm');
        return fs.readFile(wasmPath);
    } else {
        throw new Error('fetch is not defined');
    }
};

// Work around missing ClipboardEvent type
class MyClipboardEvent {}
globalThis.ClipboardEvent = MyClipboardEvent as ClipboardEvent;

afterEach(() => {
    cleanup();
});
