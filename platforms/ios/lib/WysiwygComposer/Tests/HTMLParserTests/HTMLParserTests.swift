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

@testable import HTMLParser
import XCTest

final class HTMLParserTests: XCTestCase {
    func testBuildAttributedFromHtml() throws {
        let html = "Some <strong>bold and <em>italic</em> text</strong>"
        let attributed = try HTMLParser.parse(html: html)
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
            _ = try HTMLParser.parse(html: invalidString, encoding: encoding)
        } catch {
            XCTAssertEqual(error as? BuildHtmlAttributedError, BuildHtmlAttributedError.dataError(encoding: encoding))
            XCTAssertEqual(error.localizedDescription,
                           "Unable to encode string with: \(encoding.description) rawValue: \(encoding.rawValue)")
        }
    }

    func testCodeBlockBackgroundStyleIsApplied() throws {
        let html = "<pre>code block</pre>"
        let attributed = try HTMLParser.parse(html: html)
        XCTAssertEqual(attributed.attribute(.backgroundStyle, at: 0, effectiveRange: nil) as? BackgroundStyle,
                       HTMLParserStyle.standard.codeBlockBackgroundStyle)
    }

    func testQuoteBackgroundStyleIsApplied() throws {
        let html = "<blockquote>quote</blockquote>some text"
        let attributed = try HTMLParser.parse(html: html)
        XCTAssertEqual(attributed.attribute(.backgroundStyle, at: 0, effectiveRange: nil) as? BackgroundStyle,
                       HTMLParserStyle.standard.quoteBackgroundStyle)
    }

    func testInlineCodeBackgroundColorIsApplied() throws {
        let html = "<code>inline code</code>"
        let attributed = try HTMLParser.parse(html: html)
        XCTAssertEqual(attributed.backgroundColor(at: 0),
                       HTMLParserStyle.standard.codeBackgroundColor)
    }
}
