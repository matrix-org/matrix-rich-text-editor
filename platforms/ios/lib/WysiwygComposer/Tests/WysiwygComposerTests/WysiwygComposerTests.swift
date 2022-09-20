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

// FIXME: replace ZWSP with another solution
private enum Constants {
    static let zwsp = "\u{200B}"
}

final class WysiwygComposerTests: XCTestCase {
    func testSetBaseStringWithEmoji() {
        let composer = newComposerModel()
        let update = composer.replaceText(newText: TestConstants.testStringWithEmojis)
        switch update.textUpdate() {
        case .keep, .select:
            XCTFail("Expected replace all HTML update")
        case .replaceAll(replacementHtml: let codeUnits,
                         startUtf16Codeunit: let start,
                         endUtf16Codeunit: let end):
            // Text is preserved, including emojis.
            XCTAssertEqual(String(utf16CodeUnits: codeUnits, count: codeUnits.count),
                           TestConstants.testStringWithEmojis)
            // Selection is set at the end of the text.
            XCTAssertEqual(start, end)
            XCTAssertEqual(end, 14)
        }
    }

    func testBackspacingEmoji() {
        let composer = newComposerModel()
        _ = composer.replaceText(newText: TestConstants.testStringWithEmojis)

        _ = composer.select(startUtf16Codeunit: 7, endUtf16Codeunit: 14)

        let update = composer.backspace()
        switch update.textUpdate() {
        case .keep, .select:
            XCTFail("Expected replace all HTML update")
        case .replaceAll(replacementHtml: let codeUnits,
                         startUtf16Codeunit: let start,
                         endUtf16Codeunit: let end):
            // Text should remove exactly the last emoji.
            XCTAssertEqual(String(utf16CodeUnits: codeUnits, count: codeUnits.count),
                           TestConstants.testStringAfterBackspace)
            XCTAssertEqual(start, end)
            XCTAssertEqual(start, 7)
        }
    }

    func testFormatBold() throws {
        let composer = newComposerModel()
        _ = composer.replaceText(newText: "This is bold text")
        _ = composer.select(startUtf16Codeunit: 8, endUtf16Codeunit: 12)
        let update = composer.bold()
        switch update.textUpdate() {
        case .keep, .select:
            XCTFail("Expected replace all HTML update")
        case .replaceAll(replacementHtml: let codeUnits,
                         startUtf16Codeunit: let start,
                         endUtf16Codeunit: let end):
            let html = String(utf16CodeUnits: codeUnits, count: codeUnits.count)
            XCTAssertEqual(html,
                           "This is <strong>bold</strong> text")
            // Selection is kept after format.
            XCTAssertEqual(start, 8)
            XCTAssertEqual(end, 12)
            // Constructed attributed string sets bold on the selected range.
            let attributed = try NSAttributedString(html: html)
            attributed.enumerateTypedAttribute(.font, in: .init(location: 8, length: 4)) { (font: UIFont, range, _) in
                XCTAssertEqual(range, .init(location: 8, length: 4))
                XCTAssertTrue(font.fontDescriptor.symbolicTraits.contains(.traitBold))
            }
        }

        let tree = composer.toTree()
        XCTAssertEqual(
            tree,
            """

            ├>\"This is \"
            ├>strong
            │ └>\"bold\"
            └>\" text\"

            """
        )
    }

    // swiftlint:disable:next function_body_length
    func testLists() {
        let composer = newComposerModel()
        _ = composer.orderedList()
        _ = composer.replaceText(newText: "Item 1")
        _ = composer.enter()
        _ = composer.replaceText(newText: "Item 2")
        // Add a third list item
        let update = composer.enter()
        switch update.textUpdate() {
        case .keep, .select:
            XCTFail("Expected replace all HTML update")
        case .replaceAll(replacementHtml: let codeUnits,
                         startUtf16Codeunit: let start,
                         endUtf16Codeunit: let end):
            let html = String(utf16CodeUnits: codeUnits, count: codeUnits.count)
            XCTAssertEqual(html,
                           "<ol><li>Item 1</li><li>"
                           + Constants.zwsp
                           + "Item 2</li><li>"
                           + Constants.zwsp
                           + "</li></ol>")
            XCTAssertEqual(start, end)
            XCTAssertEqual(start, 14)
        }
        // Remove it
        let update2 = composer.enter()
        switch update2.textUpdate() {
        case .keep, .select:
            XCTFail("Expected replace all HTML update")
        case .replaceAll(replacementHtml: let codeUnits,
                         startUtf16Codeunit: let start,
                         endUtf16Codeunit: let end):
            let html = String(utf16CodeUnits: codeUnits, count: codeUnits.count)
            XCTAssertEqual(html,
                           "<ol><li>Item 1</li><li>"
                           + Constants.zwsp
                           + "Item 2</li></ol>"
                           + Constants.zwsp)
            XCTAssertEqual(start, end)
            XCTAssertEqual(start, 14)
        }
        // Insert some text afterwards
        let update3 = composer.replaceText(newText: "Some text")
        switch update3.textUpdate() {
        case .keep, .select:
            XCTFail("Expected replace all HTML update")
        case .replaceAll(replacementHtml: let codeUnits,
                         startUtf16Codeunit: let start,
                         endUtf16Codeunit: let end):
            let html = String(utf16CodeUnits: codeUnits, count: codeUnits.count)
            XCTAssertEqual(html,
                           "<ol><li>Item 1</li><li>"
                           + Constants.zwsp
                           + "Item 2</li></ol>"
                           + Constants.zwsp
                           + "Some text")
            XCTAssertEqual(start, end)
            XCTAssertEqual(start, 23)
        }
    }
}
