# `wysiwyg-ffi`

Rust code that can be used to generate Kotlin and Swift bindings for
wysiwyg-rust.

## Building

* [Install Rust](https://www.rust-lang.org/tools/install)
* [Install Android NDK](https://developer.android.com/ndk/downloads) - download
  android-ndk-r22b-linux-x86_64.zip and unzip it e.g.:

```bash
unzip android-ndk-r22b-linux-x86_64.zip
```

NOTE: at time of writing (2022-06-28) you needed to use android-ndk-r22b or
earlier, because later versions fail with an error like
`ld: error: unable to find library -lgcc`.  See
[rust-lang #85806](https://github.com/rust-lang/rust/pull/85806) for more.

* Configure Rust for cross compilation:

```bash
rustup target add aarch64-linux-android
rustup target add x86_64-linux-android
```

(Note: `aarch64` is for most physical Android devices, but `x86_64` is useful
for running an Android emulator on a PC. You can also add `i686` if you use
32-bit emulators.)

* Edit Cargo config `bindings/wysiwyg-ffi/.cargo/config.toml` to contain
  something like:

```toml
[target.aarch64-linux-android]
ar = "NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/ar"
linker = "NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android30-clang"

[target.x86_64-linux-android]
ar = "NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/ar"
linker = "NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android30-clang"
```

Replacing NDK_HOME with something like `/home/andy/android-ndk-r22b/`.

(Again, add the equivalent for `i686` if you use a 32-bit emulator.)

(Note: this file is ignored in the top-level `.gitignore` file, so it won't be
checked in to Git.)

(More details in the
[Cargo reference](https://doc.rust-lang.org/cargo/reference/config.html)).

* Install uniffi-bindgen:

```bash
cargo install uniffi_bindgen
```

* Build shared object:

```bash
cd bindings/wysiwyg-ffi
cargo build --target aarch64-linux-android
cargo build --target x86_64-linux-android
```

This will create:

```
../../target/aarch64-linux-android/debug/libwysiwyg_ffi.a
../../target/aarch64-linux-android/debug/libwysiwyg_ffi.d
../../target/aarch64-linux-android/debug/libwysiwyg_ffi.so
../../target/x86_64-linux-android/debug/libwysiwyg_ffi.a
../../target/x86_64-linux-android/debug/libwysiwyg_ffi.d
../../target/x86_64-linux-android/debug/libwysiwyg_ffi.so
```

* Copy the shared libraries into your Android project:

```bash
mkdir -p ../../examples/example-android/app/src/main/jniLibs/aarch64
cp ../../target/aarch64-linux-android/debug/libwysiwyg_ffi.so \
    ../../examples/example-android/app/src/main/jniLibs/aarch64

mkdir -p ../../examples/example-android/app/src/main/jniLibs/x86_64
cp ../../target/x86_64-linux-android/debug/libwysiwyg_ffi.so \
    ../../examples/example-android/app/src/main/jniLibs/x86_64
```

Where ANDROID_PROJECT_HOME is something like
`../../../wysiwyg-android/RustFFITest`.

(The example target path above is for Element Android.)

See
[Include prebuilt native libraries](https://developer.android.com/studio/projects/gradle-external-native-builds#jniLibs)
in the Android documentation for more details.

* Generate the Kotlin and Swift bindings:

```bash
cd bindings/wysiwyg-ffi
uniffi-bindgen generate src/api.udl \
    --language kotlin \
    --out-dir ../../examples/example-android/app/build/generated/source/

# TODO
# uniffi-bindgen generate src/api.udl \
#     --language swift \
#     --out-dir ../../examples/example-kotlin/TODO
```

## Trying it out
