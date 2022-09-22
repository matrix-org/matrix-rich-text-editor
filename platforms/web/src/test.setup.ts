// We are using node api here
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-nocheck

import fs from 'fs';
import path from 'path';

globalThis.fetch = (url) => {
    // wysiwyg.js binding uses fetch to get the wasm file
    // we return manually here the wasm file
    if (url instanceof URL && url.href.includes('wysiwyg_bg.wasm')) {
        const wasmPath = path.resolve(__dirname, 'generated/wysiwyg_bg.wasm');
        return fs.readFileSync(wasmPath);
    } else {
        throw new Error('fetch is not defined');
    }
};
