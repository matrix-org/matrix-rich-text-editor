//
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

@testable import WysiwygComposer
import XCTest

final class CollectionDifferenceTests: XCTestCase {
    func testSimpleRemoval() {
        let difference = "tex".difference(from: "text")
        XCTAssertEqual(difference.removedRanges,
                       [NSRange(location: 3, length: 1)])
        XCTAssertTrue(difference.textInsertions.isEmpty)
    }

    func testMultipleRemovals() {
        let difference = "ex".difference(from: "text")
        XCTAssertEqual(difference.removedRanges,
                       [NSRange(location: 0, length: 1),
                        NSRange(location: 3, length: 1)])
        XCTAssertTrue(difference.textInsertions.isEmpty)
    }

    func testSimpleInsertion() {
        let difference = "text".difference(from: "tex")
        XCTAssertEqual(difference.textInsertions.map(\.range),
                       [NSRange(location: 3, length: 1)])
        XCTAssertEqual(difference.textInsertions.map(\.text),
                       ["t"])
        XCTAssertTrue(difference.removedRanges.isEmpty)
    }

    func testMultipleInsertions() {
        let difference = "texts".difference(from: "ex")
        XCTAssertEqual(difference.textInsertions.map(\.range),
                       [NSRange(location: 0, length: 1),
                        NSRange(location: 3, length: 2)])
        XCTAssertEqual(difference.textInsertions.map(\.text),
                       ["t", "ts"])
        XCTAssertTrue(difference.removedRanges.isEmpty)
    }

    func testSimpleReplacement() {
        let difference = "tessst".difference(from: "text")
        XCTAssertEqual(difference.removedRanges,
                       [NSRange(location: 2, length: 1)])
        XCTAssertEqual(difference.textInsertions.map(\.range),
                       [NSRange(location: 2, length: 3)])
        XCTAssertEqual(difference.textInsertions.map(\.text),
                       ["sss"])
    }

    func testMultipleReplacements() {
        let difference = "wexpf".difference(from: "text")
        XCTAssertEqual(difference.removedRanges,
                       [NSRange(location: 0, length: 1),
                        NSRange(location: 3, length: 1)])
        XCTAssertEqual(difference.textInsertions.map(\.range),
                       [NSRange(location: 0, length: 1),
                        NSRange(location: 3, length: 2)])
        XCTAssertEqual(difference.textInsertions.map(\.text),
                       ["w", "pf"])
    }
}
