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

VERSION=$1

echo "Updating Rust"
$SED_CMD "s/^version\s*=\s*\".*\"/version = \"$VERSION\"/g" bindings/wysiwyg-ffi/Cargo.toml
$SED_CMD "s/^version\s*=\s*\".*\"/version = \"$VERSION\"/g" bindings/wysiwyg-wasm/Cargo.toml
$SED_CMD "s/^version\s*=\s*\".*\"/version = \"$VERSION\"/g" crates/wysiwyg/Cargo.toml

echo "Updating Web"
$SED_CMD "s/\"version\":\s*\".*\"/\"version\": \"$VERSION\"/g" platforms/web/package.json
$SED_CMD "s/\"version\":\s*\".*\"/\"version\": \"$VERSION\"/g" bindings/wysiwyg-wasm/package.json

echo "Updating Android"
$SED_CMD "s/version\s*=\s*\".*\"/version = \"$VERSION\"/g" platforms/android/publish.gradle