# Matrix Rich Text Editor
[![codecov](https://codecov.io/gh/matrix-org/matrix-rich-text-editor/branch/main/graph/badge.svg?token=UFBR3KtPdg)](https://codecov.io/gh/matrix-org/matrix-rich-text-editor)
![GitHub](https://img.shields.io/github/license/matrix-org/matrix-rich-text-editor)

A cross-platform rich text editor intended for use in Matrix clients including
the Element clients.

Works on Web, Android and iOS using a cross-platform core written in Rust,
and platform-specific wrappers.

__Important note:__ This project is still in an early stage. Minor versions could bring 
breaking API changes, see [CHANGELOG.md](CHANGELOG.md) for details.
Bugs and crashes may occur, please report them [here](https://github.com/matrix-org/matrix-rich-text-editor/issues/new).

## Live demo

Try it out at
[matrix-org.github.io/matrix-rich-text-editor](https://matrix-org.github.io/matrix-rich-text-editor/).

## Building the code

Get the prerequisites for each platform by reading the READMEs for them:

* WASM/JavaScript:
  [bindings/wysiwyg-wasm/README.md](bindings/wysiwyg-wasm/README.md)

* Android/Kotlin or iOS/Swift:
  [bindings/wysiwyg-ffi/README.md](bindings/wysiwyg-ffi/README.md)

Now, to build all the bindings, try:

```bash
make
```

To build for a single platform, or to learn more, see the individual README
files above.

## Release the code

See [RELEASE.md](RELEASE.md).

## More info

For more detailed explanations and examples of platform-specific code to use
Rust bindings like those generated here, see
[Building cross-platform Rust for Web, Android and iOS – a minimal example](https://www.artificialworlds.net/blog/2022/07/06/building-cross-platform-rust-for-web-android-and-ios-a-minimal-example/).

## See also

* The [Browser Selections Inventory](https://gitlab.com/andybalaam/browser-selections)
  - used while writing tests, to persuade the browser to select text in the
  same way as if it had been done manually.

## License

[Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)
