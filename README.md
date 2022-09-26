# WYSIWYG-rust

Experiments with a WYSIWYG editor for Matrix, for possible inclusion in Element
clients.

## Building the code

Get the prerequisites for each platform by reading the READMEs for them:

* WASM/JavaScript:
  [bindings/wysiwyg-wasm/README.md](bindings/wysiwyg-wasm/README.md)

* Kotlin/Android:
  [bindings/wysiwyg-ffi/README.md#android](bindings/wysiwyg-ffi/README.md#android)

* Swift/iOS:
  [bindings/wysiwyg-ffi/README.md#ios](bindings/wysiwyg-ffi/README.md#ios)

Now, to build all the bindings, try:

```bash
make
```

To build for a single platform, or to learn more, see the individual README
files above.

## Release the code

* Swift/iOS:
Run `release_ios.sh` which will open a PR against [the swift package repo](https://github.com/matrix-org/matrix-wysiwyg-composer-swift) with the latest from main(in future will handle tags/releases).

## More info

For more detailed explanations and examples of platform-specific code to use
Rust bindings like those generated here, see
[Building cross-platform Rust for Web, Android and iOS – a minimal example](https://www.artificialworlds.net/blog/2022/07/06/building-cross-platform-rust-for-web-android-and-ios-a-minimal-example/).

## License

[Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)
