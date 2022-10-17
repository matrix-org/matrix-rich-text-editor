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
... plus other files
```

These files should be copied into a web project and imported with code like:

```html
<script type="module">
import init, { some_method_from_rust }
    from './generated/matrix_sdk_wysiwyg.js';

async function run() {
    await init();
    some_method_from_rust();
}

run();
</script>
```

## Profiling

To generate a debugging/profiling Wasm module, use the following command
instead of `npm run build`:

```sh
$ npm run dev-build
```