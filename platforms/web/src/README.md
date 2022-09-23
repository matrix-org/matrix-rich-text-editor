# React Matrix WYSIWYG Composer

[![react-build](https://github.com/matrix-org/matrix-wysiwyg/actions/workflows/react-build.yml/badge.svg?branch=main)](https://github.com/matrix-org/matrix-wysiwyg/actions/workflows/react-build.yml)

The Matrix WYSIWYG composer is a React library.

## TODO NPM / Usage documentation

## Contribute 

### Install

#### Generate WASM bindings

The composer uses a cross-platform rust library. In order to work as intended, the WASM bindings must be generated according to the [Matrix Wysisyg README.md](../../../README.md)

#### Yarn install

Requirements:
    - node >= 8.X
    - yarn 1.X


```sh
yarn install
```

### Build

[Vite](https://vitejs.dev/) is the Wysiwyg Composer builder.

To build:

```
yarn build
```

The builded files are located in the `dist` folder

### Testing

The tests are powered by [Vitest](https://vitest.dev/).

To run them, different options are available:

- Classic run

```sh
yarn test
```

- Watch mode

```sh
yarn test:watch
```

- Code coverage

```sh
yarn coverage
```

The coverage report is located in the `coverage` folder.