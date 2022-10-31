# React Matrix Rich Text Editor

[![react-build](https://github.com/matrix-org/matrix-wysiwyg/actions/workflows/react-build.yml/badge.svg?branch=main)](https://github.com/matrix-org/matrix-wysiwyg/actions/workflows/react-build.yml)

The Matrix Rich Text Editor is a React library.

## TODO NPM / Usage documentation

The wysiwyg composer API is a react hook.

```tsx
const { ref, isWysiwygReady, wysiwyg } = useWysiwyg();

return (
    <div>
        <button type="button" onClick={wysiwyg.bold}>
            bold
        </button>
        <div ref={ref} contentEditable={isWysiwygReady} />
    </div>
);
```

## Contribute

### Install

#### Generate WASM bindings

The composer uses a cross-platform rust library. In order to work as intended, the WASM bindings must be generated according to the [Matrix Rich Text Editor README.md](../../../README.md)

#### Yarn install

Requirements:

-   node >= 8.X
-   yarn 1.X

```sh
yarn install
```

### Dev

#### Folder structure

-   Inside the `lib` folder, the wysiwyg composer library files are located with `useWysiwyg` as en entrypoint
-   Inside the `src` folder, the demo page of the composer is located.

### Dev mode

Launch:

```sh
yarn dev
```

A dev server with hot reload is launched on `http://localhost:5173/` by default.

For more information, see [Vite](https://vitejs.dev/guide/features.html#hot-module-replacement) for more information.

### Build

[Vite](https://vitejs.dev/) is the Wysiwyg Composer builder.

To build:

```sh
yarn build
```

The builded files are located in the `dist` folder

### Testing

The tests are powered by [Vitest](https://vitest.dev/).

To run them, different options are available:

-   Classic run

```sh
yarn test
```

-   Watch mode

```sh
yarn test:watch
```

-   Code coverage

```sh
yarn coverage
```

The coverage report is located in the `coverage` folder.
