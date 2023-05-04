#!/bin/bash

set -eo pipefail

xcodebuild \
  -project Wysiwyg.xcodeproj \
  -scheme WysiwygComposerTests \
  -sdk iphonesimulator \
  -destination 'platform=iOS Simulator,name=iPhone 14,OS=16.2' \
  -derivedDataPath ./DerivedData \
  -enableCodeCoverage YES \
  test
