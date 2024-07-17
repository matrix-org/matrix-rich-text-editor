#!/bin/bash

set -eo pipefail

xcodebuild \
  -project Wysiwyg.xcodeproj \
  -scheme WysiwygComposerTests \
  -sdk iphonesimulator \
  -destination 'platform=iOS Simulator,name=iPhone 15,OS=17.5' \
  -derivedDataPath ./DerivedData \
  -resultBundlePath tests.xcresult \
  -enableCodeCoverage YES \
  test
