# `wysiwyg-ffi`

Rust code that can be used to generate Kotlin and Swift bindings for
wysiwyg-rust.

## Building

### Android

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

Make sure you install same version of uniffi_bindgen that is mentioned in
`bindings/wysiwyg-ffi/Cargo.toml`, otherwise you will get weird errors.

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

* Strip the libraries to make them smaller:

```bash
NDK_HOME/toolchains/aarch64-linux-android-4.9/prebuilt/linux-x86_64/bin/aarch64-linux-android-strip \
    ../../target/aarch64-linux-android/debug/libwysiwyg_ffi.so
NDK_HOME/toolchains/x86_64-4.9/prebuilt/linux-x86_64/bin/x86_64-linux-android-strip \
    ../../target/x86_64-linux-android/debug/libwysiwyg_ffi.so
```

Replacing NDK_HOME with something like `/home/andy/android-ndk-r22b/`.

* Copy the shared libraries into your Android project:

```bash
mkdir -p ../../examples/example-android/app/src/main/jniLibs/aarch64
cp ../../target/aarch64-linux-android/debug/libwysiwyg_ffi.so \
    ANDROID_PROJECT_HOME/app/src/main/jniLibs/aarch64

mkdir -p ../../examples/example-android/app/src/main/jniLibs/x86_64
cp ../../target/x86_64-linux-android/debug/libwysiwyg_ffi.so \
    ANDROID_PROJECT_HOME/app/src/main/jniLibs/x86_64

mkdir -p ../../examples/example-android/app/src/main/jniLibs/arm64-v8a
cp ../../target/aarch64-linux-android/debug/libwysiwyg_ffi.so \
    ANDROID_PROJECT_HOME/app/src/main/jniLibs/arm64-v8a
```

Where ANDROID_PROJECT_HOME is the root folder of an Android Studio project.

In your Android project's app/build.gradle you need a section like this:

```gradle
android.applicationVariants.all { variant ->
    def t = tasks.register("generate${variant.name.capitalize()}UniFFIBindings", Exec) {
        workingDir "${project.projectDir}"
        // Runs the bindings generation, note that you must have uniffi-bindgen installed and in your PATH environment variable
        commandLine 'uniffi-bindgen', 'generate', '../../../bindings/ffi/src/wysiwyg_composer.udl', '--language', 'kotlin', '--out-dir', "${buildDir}/generated/source/uniffi/${variant.name}/java"
    }
    variant.javaCompileProvider.get().dependsOn(t)
    def sourceSet = variant.sourceSets.find { it.name == variant.name }
    sourceSet.java.srcDir new File(buildDir, "generated/source/uniffi/${variant.name}/java")
}
```

which will then allow you to use the Rust code in your Kotlin like this:

```kotlin
val y = uniffi.wysiwyg_composer.someMethod()
```

See
[Example Rust Bindings](https://gitlab.com/andybalaam/example-rust-bindings/)
for a full example of how it all fits together.

### iOS

* Build shared object:

```bash
cd bindings/ffi
cargo build --release --target aarch64-apple-ios
cargo build --release --target aarch64-apple-ios-sim
cargo build --release --target x86_64-apple-ios

mkdir -p ../../target/ios-simulator
lipo -create \
  ../../target/x86_64-apple-ios/release/libwysiwyg_ffi.a \
  ../../target/aarch64-apple-ios-sim/release/libwysiwyg_ffi.a \
  -output ../../target/ios-simulator/libwysiwyg_ffi.a
```

* This will create static libraries for both iOS devices and simulators:

```
../../target/x86_64-apple-ios/debug/libwysiwyg_ffi.a
../../target/ios-simulator/libwysiwyg_ffi.a
```

* Generate the bindings inside given output dir:

⚠️ The installed version should always match the version used by Cargo, see `Cargo.toml` content inside this directory to retrieve it and use `cargo install uniffi_bindgen --version <VERSION_NUMBER>` if needed. 

```bash
uniffi-bindgen \
    generate src/wysiwyg_composer.udl \
    --language swift \
    --config uniffi.toml \
    --out-dir MY_OUTPUT_DIR

cd MY_OUTPUT_DIR
mv *.h         headers/
mv *.modulemap headers/module.modulemap
mv *.swift     Sources/
```

Note: The project should always have a single Swift modulemap file, and it should be named `module.modulemap` otherwise the generated framework will not expose symbols properly to Swift.

* Generate the .xcframework file:

```bash
xcodebuild -create-xcframework \
  -library ../../target/aarch64-apple-ios/debug/libwysiwyg_composer.a" \
  -headers MY_OUTPUT_DIR/headers \
  -library MY_OUTPUT_DIR/libwysiwyg_composer.a" \
  -headers MY_OUTPUT_DIR/headers \
  -output MY_OUTPUT_DIR/ExampleRustBindings.xcframework"
```

Now you can use the framework wherever you see fit, just add the framework (as well as the generated Swift code from `MY_OUTPUT_DIR/Sources/`) as a dependency inside an existing XCode project, or package them with your preferred dependency manager.  
See [Example Rust Bindings](https://gitlab.com/andybalaam/example-rust-bindings/)
for a full example of how it all fits together.
