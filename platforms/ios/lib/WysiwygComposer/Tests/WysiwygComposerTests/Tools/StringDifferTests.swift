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
        let replacement = try StringDiffer.replacement(from: identicalText, to: identicalText)
        XCTAssertNil(replacement)
    }

    func testSimpleRemoval() throws {
        let replacement = try StringDiffer.replacement(from: "text", to: "te")
        XCTAssertEqual(replacement?.range,
                       NSRange(location: 2, length: 2))
        XCTAssertEqual(replacement?.text, "")
    }

    func testSimpleInsertion() throws {
        let replacement = try StringDiffer.replacement(from: "te", to: "text")
        XCTAssertEqual(replacement?.range,
                       NSRange(location: 2, length: 0))
        XCTAssertEqual(replacement?.text,
                       "xt")
    }

    func testFullReplacement() throws {
        let oldText = "wa"
        let newText = "わ"
        let replacement = try StringDiffer.replacement(from: oldText, to: newText)
        XCTAssertEqual(replacement?.range,
                       NSRange(location: 0, length: 2))
        XCTAssertEqual(replacement?.text, "わ")
    }

    func testPartialReplacement() throws {
        let oldText = "わta"
        let newText = "わた"
        let replacement = try StringDiffer.replacement(from: oldText, to: newText)
        XCTAssertEqual(replacement?.range,
                       NSRange(location: 1, length: 2))
        XCTAssertEqual(replacement?.text, "た")
    }

    func testDoubleReplacementIsNotHandled() throws {
        let oldText = "text"
        let newText = "fexf"
        XCTAssertThrowsError(try StringDiffer.replacement(from: oldText, to: newText), "doubleReplacementIsNotHandled") { error in
            XCTAssertEqual(error as? StringDifferError,
                           StringDifferError.tooComplicated)
        }
    }

    func testInsertionsDontMatchRemovalsLocation() throws {
        let oldText = "text"
        let newText = "extab"
        XCTAssertThrowsError(try StringDiffer.replacement(from: oldText, to: newText), "insertionsDontMatchRemovalsLocation") { error in
            XCTAssertEqual(error as? StringDifferError,
                           StringDifferError.insertionsDontMatchRemovals)
        }
    }
}
