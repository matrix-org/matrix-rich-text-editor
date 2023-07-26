#!/usr/bin/env bash

GENERATION_PATH=.generated/ios

UDL_FILE_PATH=bindings/wysiwyg-ffi/src/wysiwyg_composer.udl
UNIFFI_CONFIG_FILE_PATH=bindings/wysiwyg-ffi/uniffi.toml

ARM64_LIB_PATH=target/aarch64-apple-ios/release/libuniffi_wysiwyg_composer.a
ARM64_SIM_LIB_PATH=target/aarch64-apple-ios-sim/release/libuniffi_wysiwyg_composer.a
X86_LIB_PATH=target/x86_64-apple-ios/release/libuniffi_wysiwyg_composer.a
SIM_LIB_PATH=target/ios-simulator/libuniffi_wysiwyg_composer.a

IOS_PATH=platforms/ios
TOOLS_PATH="${IOS_PATH}/tools"

SWIFT_PACKAGE_PATH="${IOS_PATH}/lib/WysiwygComposer"
SWIFT_BINDINGS_FILE_PATH="${SWIFT_PACKAGE_PATH}/Sources/WysiwygComposer/WysiwygComposer.swift"

XCFRAMEWORK_PATH="${SWIFT_PACKAGE_PATH}/WysiwygComposerFFI.xcframework"
XCFRAMEWORK_SIM_PATH="${XCFRAMEWORK_PATH}/ios-arm64_x86_64-simulator/WysiwygComposerFFI.framework"
XCFRAMEWORK_SIM_HEADERS_PATH="${XCFRAMEWORK_SIM_PATH}/Headers"
XCFRAMEWORK_SIM_MODULES_PATH="${XCFRAMEWORK_SIM_PATH}/Modules"
XCFRAMEWORK_SIM_LIBRARY_PATH="${XCFRAMEWORK_SIM_PATH}/WysiwygComposerFFI"
XCFRAMEWORK_ARM64_PATH="${XCFRAMEWORK_PATH}/ios-arm64/WysiwygComposerFFI.framework"
XCFRAMEWORK_ARM64_HEADERS_PATH="${XCFRAMEWORK_ARM64_PATH}/Headers"
XCFRAMEWORK_ARM64_MODULES_PATH="${XCFRAMEWORK_ARM64_PATH}/Modules"
XCFRAMEWORK_ARM64_LIBRARY_PATH="${XCFRAMEWORK_ARM64_PATH}/WysiwygComposerFFI"

# Build libraries for each platform
cargo build -p uniffi-wysiwyg-composer --release --target aarch64-apple-ios
cargo build -p uniffi-wysiwyg-composer --release --target aarch64-apple-ios-sim
cargo build -p uniffi-wysiwyg-composer --release --target x86_64-apple-ios

# Merge x86 and simulator arm libraries with lipo
mkdir -p target/ios-simulator
lipo -create $X86_LIB_PATH $ARM64_SIM_LIB_PATH -output $SIM_LIB_PATH

# Remove previous artefacts and files
rm -rf $XCFRAMEWORK_PATH
rm -f $SWIFT_BINDINGS_FILE_PATH
rm -rf $GENERATION_PATH

# Generate headers & Swift bindings
mkdir -p $GENERATION_PATH
cargo uniffi-bindgen generate --library $ARM64_LIB_PATH -l swift --out-dir $GENERATION_PATH

# Move Swift file to expected location
#
# Note: we use sed to tweak the generated Swift bindings and catch Rust panics, 
# this should be removed when the Rust code is 100% safe (see `ComposerModelWrapper.swift`).
mv "${GENERATION_PATH}/WysiwygComposer.swift" $SWIFT_BINDINGS_FILE_PATH
sed -i "" -e '1h;2,$H;$!d;g' -e 's/) -> ComposerUpdate {\n        return try! FfiConverterTypeComposerUpdate.lift(\n            try!/) throws -> ComposerUpdate {\n        return try FfiConverterTypeComposerUpdate.lift(\n            try/g' $SWIFT_BINDINGS_FILE_PATH
sed -i "" -e '1h;2,$H;$!d;g' -e 's/) -> ComposerUpdate/) throws -> ComposerUpdate/g' $SWIFT_BINDINGS_FILE_PATH

# Create xcframework hierarchy
mkdir -p $XCFRAMEWORK_SIM_HEADERS_PATH
mkdir $XCFRAMEWORK_SIM_MODULES_PATH
mkdir -p $XCFRAMEWORK_ARM64_HEADERS_PATH
mkdir $XCFRAMEWORK_ARM64_MODULES_PATH

# Copy/move files to expected locations
#
# Note: this and the hierarchy created above are actually
# replacing the call to xcodebuild's create-xcframework because
# it doesn't build up the hierarchy in a way that would avoid
# conflicts between multiple Rust libraries imported into the same
# hosting application. This does, because .framework objects
# have their own directory in DerivedData, whereas root headers
# directory module.modulemap files tend to conflict with each other
# as Xcode blindly moves them all to the same include folder.
mv $ARM64_LIB_PATH $XCFRAMEWORK_ARM64_LIBRARY_PATH
mv $SIM_LIB_PATH $XCFRAMEWORK_SIM_LIBRARY_PATH
cp ${GENERATION_PATH}/*.h $XCFRAMEWORK_SIM_HEADERS_PATH
mv ${GENERATION_PATH}/*.h $XCFRAMEWORK_ARM64_HEADERS_PATH
cp "${TOOLS_PATH}/Info.plist" $XCFRAMEWORK_PATH
cp "${TOOLS_PATH}/module.modulemap" $XCFRAMEWORK_SIM_MODULES_PATH
cp "${TOOLS_PATH}/module.modulemap" $XCFRAMEWORK_ARM64_MODULES_PATH
