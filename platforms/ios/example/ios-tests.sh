#!/bin/bash

set -eo pipefail

xcodebuild \
  -project Wysiwyg.xcodeproj \
  -scheme Wysiwyg \
  -sdk iphonesimulator \
  -destination 'platform=iOS Simulator,name=iPhone 13,OS=15.5' \
  -derivedDataPath ./DerivedData \
  test
