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
        case let .replaceAll(replacementHtml: codeUnits,
                             startUtf16Codeunit: start,
                             endUtf16Codeunit: end):
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
        case let .replaceAll(replacementHtml: codeUnits,
                             startUtf16Codeunit: start,
                             endUtf16Codeunit: end):
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
        case let .replaceAll(replacementHtml: codeUnits,
                             startUtf16Codeunit: start,
                             endUtf16Codeunit: end):
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

    func testLists() {
        let composer = newComposerModel()
        _ = composer.orderedList()
        _ = composer.replaceText(newText: "Item 1")
        _ = composer.enter()
        _ = composer.replaceText(newText: "Item 2")
        // Add a thirs list item
        _ = composer.enter()
        XCTAssertEqual(composer.getContentAsHtml(),
                       "<ol><li>"
                           + Constants.zwsp
                           + "Item 1</li><li>"
                           + Constants.zwsp
                           + "Item 2</li><li>"
                           + Constants.zwsp
                           + "</li></ol>")
        XCTAssertEqual(composer.getCurrentDomState().start, composer.getCurrentDomState().end)
        XCTAssertEqual(composer.getCurrentDomState().start, 15)
        // Remove it
        _ = composer.enter()
        XCTAssertEqual(composer.getContentAsHtml(),
                       "<ol><li>"
                           + Constants.zwsp
                           + "Item 1</li><li>"
                           + Constants.zwsp
                           + "Item 2</li></ol>"
                           + Constants.zwsp)
        XCTAssertEqual(composer.getCurrentDomState().start, composer.getCurrentDomState().end)
        XCTAssertEqual(composer.getCurrentDomState().start, 15)
        // Insert some text afterwards
        _ = composer.replaceText(newText: "Some text")
        XCTAssertEqual(composer.getContentAsHtml(),
                       "<ol><li>"
                           + Constants.zwsp
                           + "Item 1</li><li>"
                           + Constants.zwsp
                           + "Item 2</li></ol>"
                           + Constants.zwsp
                           + "Some text")
        XCTAssertEqual(composer.getCurrentDomState().start, composer.getCurrentDomState().end)
        XCTAssertEqual(composer.getCurrentDomState().start, 24)
    }
    
    func testCreateWithTextLinkAction() {
        let composer = newComposerModel()
        let action = composer.getLinkAction()
        XCTAssertEqual(action, .createWithText)
    }
    
    func testCreateLinkAction() {
        let composer = newComposerModel()
        _ = composer.replaceText(newText: "test")
        _ = composer.select(startUtf16Codeunit: 0, endUtf16Codeunit: 4)
        let action = composer.getLinkAction()
        XCTAssertEqual(action, .create)
    }
    
    func testEditLinkAction() {
        let link = "test_url"
        let composer = newComposerModel()
        _ = composer.setLinkWithText(link: link, text: "test")
        let action = composer.getLinkAction()
        XCTAssertEqual(action, .edit(link: link))
    }
    
    func testSetLinkWithText() {
        let composer = newComposerModel()
        _ = composer.setLinkWithText(link: "link", text: "text")
        XCTAssertEqual(
            composer.toTree(),
            """
            
            └>a \"link\"
              └>\"text\"
            
            """
        )
    }
    
    func testSetLink() {
        let composer = newComposerModel()
        _ = composer.replaceText(newText: "text")
        _ = composer.select(startUtf16Codeunit: 0, endUtf16Codeunit: 4)
        _ = composer.setLink(link: "link")
        XCTAssertEqual(
            composer.toTree(),
            """
            
            └>a \"link\"
              └>\"text\"
            
            """
        )
    }
    
    func testRemoveLinks() {
        let composer = newComposerModel()
        _ = composer.setLinkWithText(link: "link", text: "text")
        XCTAssertEqual(
            composer.toTree(),
            """
            
            └>a \"link\"
              └>\"text\"
            
            """
        )
        _ = composer.removeLinks()
        XCTAssertEqual(
            composer.toTree(),
            """
            
            └>"text"
            
            """
        )
    }
    
    func testInlineCode() {
        let composer = newComposerModel()
        _ = composer.inlineCode()
        _ = composer.replaceText(newText: "code")
        XCTAssertEqual(
            composer.toTree(),
            """
            
            └>code
              └>\"code\"
            
            """
        )
    }
    
    func testInlineCodeWithFormatting() {
        let composer = newComposerModel()
        _ = composer.bold()
        _ = composer.replaceText(newText: "bold")
        // This should get ignored
        _ = composer.italic()
        _ = composer.inlineCode()
        _ = composer.replaceText(newText: "code")
        print(composer.toTree())
        XCTAssertEqual(
            composer.toTree(),
            """
            
            ├>strong
            │ └>"bold"
            └>code
              └>"code"
            
            """
        )
    }
}
