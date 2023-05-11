#!/bin/sh

DIR=$(dirname -- "$0")
cd $DIR
./gradlew unitTestsWithCoverage
./gradlew instrumentationTestsWithCoverage
./gradlew generateCoverageReport
open build/reports/jacoco/generateCoverageReport/html/index.html
