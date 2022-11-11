#!/usr/bin/env bash

if (($# != 1)); then
  echo "There should be a single version argument passed."
  exit 1
fi

if [[ "$OSTYPE" == "darwin"* ]]; then
  if ! command -v gsed &> /dev/null; then
    echo "GNU-SED not found. Please install it using `brew install gnu-sed`."
    exit 1
  fi
  SED_CMD='gsed -i'
else
  SED_CMD="sed -i"
fi

SCRIPT_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"

VERSION=$1
CARGO_REGEX="s/^version\s*=\s*\".*\"/version = \"$VERSION\"/g"
PACKAGE_JSON_REGEX="s/\"version\":\s*\".*\"/\"version\": \"$VERSION\"/g"
GRADLE_PROPERTIES_REGEX="s/^VERSION_NAME=.*$/VERSION_NAME=$VERSION/g"

echo "Updating Rust"
$SED_CMD "$CARGO_REGEX" $SCRIPT_PATH/bindings/wysiwyg-ffi/Cargo.toml
$SED_CMD "$CARGO_REGEX" $SCRIPT_PATH/bindings/wysiwyg-wasm/Cargo.toml
$SED_CMD "$CARGO_REGEX" $SCRIPT_PATH/crates/wysiwyg/Cargo.toml

echo "Updating Web"
$SED_CMD "$PACKAGE_JSON_REGEX" $SCRIPT_PATH/platforms/web/package.json
$SED_CMD "$PACKAGE_JSON_REGEX" $SCRIPT_PATH/bindings/wysiwyg-wasm/package.json

echo "Updating Android"
$SED_CMD "$GRADLE_PROPERTIES_REGEX" $SCRIPT_PATH/platforms/android/gradle.properties
