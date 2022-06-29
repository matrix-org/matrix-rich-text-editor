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
```

* Edit Cargo config `bindings/wysiwyg-ffi/.cargo/config.toml` to contain
  something like:

```toml
[target.aarch64-linux-android]
ar = "NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/ar"
linker = "NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android30-clang"
```

Replacing NDK_HOME with something like `/home/andy/android-ndk-r22b/`.

(Note: this file is ignored in the top-level `.gitignore` file, so it won't be
checked in to Git.)

(More details in the
[Cargo reference](https://doc.rust-lang.org/cargo/reference/config.html)).


* Build:

```bash
cd bindings/wysiwyg-ffi
cargo build --target aarch64-linux-android
```

This will create:

```
../../target/aarch64-linux-android/debug/libwysiwyg_ffi.a
../../target/aarch64-linux-android/debug/libwysiwyg_ffi.d
../../target/aarch64-linux-android/debug/libwysiwyg_ffi.so
```

* Copy the shared library into your Android project:

```bash
cp ../../target/aarch64-linux-android/libwysiwyg_ffi.so \
    ANDROID_PROJECT_HOME/app/src/main/jniLibs/aarch64/
```

Where ANDROID_PROJECT_HOME is something like
`../../../wysiwyg-android/RustFFITest`.

(The example target path above is for Element Android.)


See
[Include prebuilt native libraries](https://developer.android.com/studio/projects/gradle-external-native-builds#jniLibs)
in the Android documentation for more details.

## Trying it out
