#!/bin/bash

set -eo pipefail

xcodebuild \
  -project Wysiwyg.xcodeproj \
  -scheme Wysiwyg \
  -sdk iphonesimulator \
  -destination 'platform=iOS Simulator,name=iPhone 15,OS=17.2' \
  -derivedDataPath ./DerivedData \
  -resultBundlePath ui-tests.xcresult \
  -enableCodeCoverage YES \
  test
  
