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
    ANDROID_PROJECT_HOME/app/src/main/jniLibs/aarch64

mkdir -p ../../examples/example-android/app/src/main/jniLibs/x86_64
cp ../../target/x86_64-linux-android/debug/libwysiwyg_ffi.so \
    ANDROID_PROJECT_HOME/app/src/main/jniLibs/x86_64
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
cargo build --target aarch64-apple-ios
cargo build --target aarch64-apple-ios-sim
cargo build --target x86_64-apple-ios

mkdir -p ../../target/ios-combined
lipo -create \
  ../../target/x86_64-apple-ios/debug/libwysiwyg_composer.a \
  ../../target/aarch64-apple-ios-sim/debug/libwysiwyg_composer.a \
  -output ../../target/ios-combined/libwysiwyg_composer.a
```

This will create:

```
../../target/x86_64-apple-ios/debug/libwysiwyg_composer.a
../../target/aarch64-apple-ios-sim/debug/libwysiwyg_composer.a
../../target/ios-combined/libwysiwyg_composer.a
```

* Copy the shared object into your XCode project

```bash
cp ../../target/ios-combined/libwysiwyg_composer.a MY_XCODE_PROJECT/
```

(Where MY_XCODE_PROJECT is the location of the XCode project.)

* Generate the bindings:

```bash
uniffi-bindgen \
    generate src/wysiwyg_composer.udl \
    --language swift \
    --config uniffi.toml \
    --out-dir MY_XCODE_PROJECT

cd MY_XCODE_PROJECT
mv *.h         headers/
mv *.modulemap headers/
mv *.swift     Sources/
```

* Generate the .xcframework file:

```bash
xcodebuild -create-xcframework \
  -library ../../target/aarch64-apple-ios/debug/libwysiwyg_composer.a" \
  -headers MY_XCODE_PROJECT/headers \
  -library MY_XCODE_PROJECT/libwysiwyg_composer.a" \
  -headers MY_XCODE_PROJECT/headers \
  -output MY_XCODE_PROJECT/ExampleRustBindings.xcframework"
```

Now you can use the code in your XCode project.  See
[Example Rust Bindings](https://gitlab.com/andybalaam/example-rust-bindings/)
for a full example of how it all fits together.
