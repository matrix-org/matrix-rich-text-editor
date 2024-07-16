#!/usr/bin/env bash

GENERATION_PATH=.generated/ios

ARM64_LIB_PATH=target/aarch64-apple-ios/release/libuniffi_wysiwyg_composer.a
ARM64_SIM_LIB_PATH=target/aarch64-apple-ios-sim/release/libuniffi_wysiwyg_composer.a
X86_LIB_PATH=target/x86_64-apple-ios/release/libuniffi_wysiwyg_composer.a
SIM_LIB_PATH=target/ios-simulator/libuniffi_wysiwyg_composer.a

IOS_PATH=platforms/ios

SWIFT_PACKAGE_PATH="${IOS_PATH}/lib/WysiwygComposer"
SWIFT_BINDINGS_FILE_PATH="${SWIFT_PACKAGE_PATH}/Sources/WysiwygComposer/WysiwygComposer.swift"

XCFRAMEWORK_PATH="${SWIFT_PACKAGE_PATH}/WysiwygComposerFFI.xcframework"

# Build libraries for all platforms
cargo build -p uniffi-wysiwyg-composer --release --target aarch64-apple-ios --target aarch64-apple-ios-sim --target x86_64-apple-ios

# Merge x86 and simulator arm libraries with lipo
mkdir -p target/ios-simulator
lipo -create $X86_LIB_PATH $ARM64_SIM_LIB_PATH -output $SIM_LIB_PATH

# Remove previous artefacts and files
rm -rf $XCFRAMEWORK_PATH
rm -f $SWIFT_BINDINGS_FILE_PATH
rm -rf $GENERATION_PATH

# Generate headers & Swift bindings
#
# Note: swiftformat is automatically run by uniffi-bindgen if available
# and mandatory for the `sed` tweaks below to work properly.
if ! command -v swiftformat &> /dev/null
then
    echo "swiftformat could not be found"
    exit 1
fi
mkdir -p $GENERATION_PATH
cargo uniffi-bindgen generate --library $ARM64_LIB_PATH -l swift --out-dir $GENERATION_PATH

# Move Swift file to expected location
#
# Note: we use sed to tweak the generated Swift bindings and catch Rust panics, 
# this should be removed when the Rust code is 100% safe (see `ComposerModelWrapper.swift`).
mv "${GENERATION_PATH}/WysiwygComposer.swift" $SWIFT_BINDINGS_FILE_PATH
sed -i "" -e '1h;2,$H;$!d;g' -e 's/) -> ComposerUpdate {\n        return try! FfiConverterTypeComposerUpdate.lift(\n            try!/) throws -> ComposerUpdate {\n        return try FfiConverterTypeComposerUpdate.lift(\n            try/g' $SWIFT_BINDINGS_FILE_PATH
sed -i "" -e '1h;2,$H;$!d;g' -e 's/) -> ComposerUpdate/) throws -> ComposerUpdate/g' $SWIFT_BINDINGS_FILE_PATH

# Making this directory is required to not have conflicts with other FFI generated xcframeworks.
mkdir $GENERATION_PATH/WysiwygComposerFFI
mv ${GENERATION_PATH}/WysiwygComposerFFI.modulemap ${GENERATION_PATH}/WysiwygComposerFFI/module.modulemap
mv ${GENERATION_PATH}/*.h ${GENERATION_PATH}/WysiwygComposerFFI
xcodebuild -create-xcframework -library $ARM64_LIB_PATH -headers $GENERATION_PATH -library $SIM_LIB_PATH -headers $GENERATION_PATH -output $XCFRAMEWORK_PATH
