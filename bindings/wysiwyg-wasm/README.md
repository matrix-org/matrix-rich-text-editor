# `wysiwyg-wasm`

WASM/JavaScript bindings for wysiwyg-rust.

## Building

* [Install Rust](https://www.rust-lang.org/tools/install)
* [Install NodeJS and NPM](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm)
* [Install wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
* Run:

```sh
cd bindings/wysiwyg-wasm
npm install
npm run build
#npm run test (no tests yet)
```

This will generate:

```
pkg/matrix_sdk_wysiwyg_bg.wasm
pkg/matrix_sdk_wysiwyg_bg.wasm.d.ts
pkg/matrix_sdk_wysiwyg.d.ts
pkg/matrix_sdk_wysiwyg.js
pkg/package.json
... plus other files copied in from the project root
```

## Trying it out

Create a file inside `pkg/`, called `try_it.mjs` like this:

```javascript
import { new_composer_model } from './wysiwyg.js';

const m = new_composer_model();
console.log(m.identify_thyself());
```

Run it like this:

```bash
$ node ./try_it.js
I am a ComposerModel
```
