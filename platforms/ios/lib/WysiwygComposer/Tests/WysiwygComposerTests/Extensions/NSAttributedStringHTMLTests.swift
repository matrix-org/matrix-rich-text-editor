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

import XCTest
@testable import WysiwygComposer

final class NSAttributedStringHtmlTests: XCTestCase {
    func testBuildAttributedFromHtml() throws {
        let html = "Some <strong>bold and <em>italic</em> text</strong>"
        let attributed = try NSAttributedString(html: html)
        XCTAssertEqual(attributed.string,
                       "Some bold and italic text")
        // Font at index 6 is bold
        let fontTraits1 = attributed.fontSymbolicTraits(at: 6)
        XCTAssert(fontTraits1.contains(.traitBold))
        XCTAssert(!fontTraits1.contains(.traitItalic))
        // Font at index 15 is bold and italic
        let fontTraits2 = attributed.fontSymbolicTraits(at: 15)
        XCTAssert(fontTraits2.isSuperset(of: [.traitBold, .traitItalic]))
        // Font at index 2 is neither italic, nor bold
        let fontTraits3 = attributed.fontSymbolicTraits(at: 2)
        XCTAssert(fontTraits3.isDisjoint(with: [.traitBold, .traitItalic]))
    }

    func testInvalidEncodingString() throws {
        let invalidString = "\u{F023}"
        let encoding = String.Encoding.ascii
        do {
            _ = try NSAttributedString(html: invalidString, encoding: encoding)
        } catch {
            XCTAssertEqual(error as? BuildHtmlAttributedError, BuildHtmlAttributedError.dataError(encoding: encoding))
            XCTAssertEqual(error.localizedDescription,
                           "Unable to encode string with: \(encoding.description) rawValue: \(encoding.rawValue)")
        }
    }
}
