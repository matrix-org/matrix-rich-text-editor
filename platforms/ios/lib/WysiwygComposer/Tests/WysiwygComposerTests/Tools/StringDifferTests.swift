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

    func testDifferentWhitespacesAreEquivalent() throws {
        let whitespaceCodeUnits = CharacterSet.whitespaces.codePoints()
        let whitespaceString = String(
            String(utf16CodeUnits: whitespaceCodeUnits, count: whitespaceCodeUnits.count)
                // We need to remove unicode characters that are related to whitespaces but have a property `White_space = no`
                .filter(\.isWhitespace)
        )
        XCTAssertNil(try StringDiffer.replacement(from: whitespaceString,
                                                  to: String(repeating: "\u{00A0}", count: whitespaceString.utf16Length)))
    }
}

private extension CharacterSet {
    func codePoints() -> [UInt16] {
        var result: [Int] = []
        var plane = 0
        for (i, w) in bitmapRepresentation.enumerated() {
            let k = i % 8193
            if k == 8192 {
                plane = Int(w) << 13
                continue
            }
            let base = (plane + k) << 3
            for j in 0..<8 where w & 1 << j != 0 {
                result.append(base + j)
            }
        }
        return result.map { UInt16($0) }
    }
}
