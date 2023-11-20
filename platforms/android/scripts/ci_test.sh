#!/bin/bash

./gradlew unitTestsWithCoverage $CI_GRADLE_ARG_PROPERTIES
./gradlew generateUnitTestCoverageReport $CI_GRADLE_ARG_PROPERTIES

# Don't exit immediately from UI test failure to collect screenshots
set +e 

./gradlew instrumentationTestsWithCoverage $CI_GRADLE_ARG_PROPERTIES

UI_TEST_EXIT_CODE=$?
if [ $UI_TEST_EXIT_CODE -ne 0 ]; then
    echo "UI tests failed."
    echo "Pulling screenshots from device..."
    adb shell ls /sdcard/Pictures/UiTest/
    mkdir build/reports/screenshots
    adb pull /sdcard/Pictures/UiTest/ build/reports/screenshots/
    exit $UI_TEST_EXIT_CODE
fi
set -e

./gradlew generateInstrumentationTestCoverageReport $CI_GRADLE_ARG_PROPERTIES

