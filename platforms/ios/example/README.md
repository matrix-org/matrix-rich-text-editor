# WYSIWYG iOS example app

This application provides an example usage of the WysiwygComposer 
package found at `/../lib/WysiwygComposer`.
It also contains UI tests validating components provided by the package.

# Setup

All that is required is to run `make ios` in the repository main folder
in order to build uniffi bindings for Swift, then the app will build
directly from XCode.

# Enable SwiftLint & SwiftFormat

SwiftLint and SwiftFormat can be enabled on both the library and the
example application from this project. The only requirement is to run `brew bundle` inside this folder, then both tools will run anytime the
example app is built.

# Gather global code coverage for WysiwygComposer.

The main scheme `Wysiwyg` provides an associated test suite that run both
WysiwygComposer's Unit tests, as well as the example app UI tests. Merged 
test results and code coverage can be used directly within XCode or 
retrieved inside DerivedData's `Logs/Test` folder for usage with an 
external tool.

You can run this test suite from terminal with this kind of command:

```bash
xcodebuild \
  -project Wysiwyg.xcodeproj \
  -scheme Wysiwyg \
  -sdk iphonesimulator \
  -destination 'platform=iOS Simulator,name=iPhone 13,OS=15.5' \
  -derivedDataPath ./DerivedData \
  test
```