# WYSIWYG-rust

Experiments with a WYSIWYG editor for Matrix, for possible inclusion in Element
clients.

## Live demo

Try it out at
[matrix-org.github.io/matrix-wysiwyg](https://matrix-org.github.io/matrix-wysiwyg/).

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
