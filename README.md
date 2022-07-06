# WYSIWYG-rust

Model code and bindings to power a WYSIWYG editor for Matrix.

## Building the code

* [Install Rust](https://www.rust-lang.org/tools/install)

```bash
cargo test
```

## Building the bindings

`Makefile` contains all the commands you need to run to get set up on
Ubuntu, Debian and similar distros:

```bash
make prerequisites
```

Now, to build all the bindings (on most Linux distros, hopefully), try:

```bash
make
```

Alternatively, to build for a single platform, or to learn more, see the
individual README files:

* WASM/JavaScript:
  [bindings/wysiwyg-wasm/README.md](bindings/wysiwyg-wasm/README.md)

* Kotlin/Android:
  [bindings/wysiwyg-ffi/README.md](bindings/wysiwyg-ffi/README.md)

## License

[Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)
