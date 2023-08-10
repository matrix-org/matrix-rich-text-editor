# Matrix Rich Text Editor Android

![Latest release badge](https://img.shields.io/badge/dynamic/xml?url=https%3A%2F%2Fs01.oss.sonatype.org%2Fcontent%2Fgroups%2Fpublic%2Fio%2Felement%2Fandroid%2Fwysiwyg%2Fmaven-metadata.xml&query=%2Fmetadata%2Fversioning%2Fversions%2Fversion%5Blast()%5D&label=Latest%20release)

## Usage

```kotlin
// Base library
implementation("io.element.android:wysiwyg:$version")

// Compose support
implementation("io.element.android:wysiwyg-compose:$version")
```

## Examples

There are two example apps that show how to build an app using this library:

- `<project>/platforms/android/example-compose` contains an example app which shows how to integrate the editor into a Jetpack Compose based app.
- `<project>/platforms/android/example-view` contains an example app which shows how to integrate the editor into a traditional View / XML layout based app.

## Troubleshooting

### UnsatisfiedLinkError
When the bindings are generated, they point to the wrong library name:

```kotlin
return "uniffi_wysiwyg_composer"
```

We can either modify the Cargo setup to generate a library with that name, or modify the generated bindings:

```kotlin
return "wysiwyg_ffi"
```

### Can't find function `some_mangled_name_2345_my_class_new()`

You probably need to re-generate the bindings.

### Cargo crashing while trying to generate the bindings

As mentioned in [wysiwyg-ffi's README](../../bindings/wysiwyg-ffi/README.md), you might need to specify the location of the NDK you want to use by adding `.cargo/config.toml` files with the right contents. 
