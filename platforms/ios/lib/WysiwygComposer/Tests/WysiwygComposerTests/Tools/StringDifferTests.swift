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

final class StringDifferTests: XCTestCase {
    func testNoReplacement() throws {
        let identicalText = "text"
        XCTAssertNil(try StringDiffer.replacement(from: identicalText, to: identicalText))
    }

    func testSimpleRemoval() throws {
        XCTAssertEqual(try StringDiffer.replacement(from: "text", to: "te"),
                       .init(location: 2, length: 2, text: ""))
    }

    func testSimpleInsertion() throws {
        XCTAssertEqual(try StringDiffer.replacement(from: "te", to: "text"),
                       .init(location: 2, length: 0, text: "xt"))
    }

    func testFullReplacement() throws {
        XCTAssertEqual(try StringDiffer.replacement(from: "wa", to: "わ"),
                       .init(location: 0, length: 2, text: "わ"))
    }

    func testPartialReplacement() throws {
        XCTAssertEqual(try StringDiffer.replacement(from: "わta", to: "わた"),
                       .init(location: 1, length: 2, text: "た"))
    }

    func testDoubleReplacementIsNotHandled() throws {
        XCTAssertThrowsError(try StringDiffer.replacement(from: "text", to: "fexf"), "doubleReplacementIsNotHandled") { error in
            XCTAssertEqual(error as? StringDifferError,
                           StringDifferError.tooComplicated)
        }
    }

    func testInsertionsDontMatchRemovalsLocation() throws {
        XCTAssertThrowsError(try StringDiffer.replacement(from: "text", to: "extab"), "insertionsDontMatchRemovalsLocation") { error in
            XCTAssertEqual(error as? StringDifferError,
                           StringDifferError.insertionsDontMatchRemovals)
        }
    }
}
