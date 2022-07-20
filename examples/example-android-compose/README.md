# WYSIWYG Android Compose Sample

## NOT WORKING AT THE MOMENT.

Cause: https://issuetracker.google.com/issues/199768107

## Possible issues:

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
