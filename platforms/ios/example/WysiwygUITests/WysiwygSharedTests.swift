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

/// Defines tests that can be shared between the SwiftUI and the UIKit example implementations.
final class WysiwygSharedTests {
    private init() { }

    /// Focus the composer text view inside given app and
    /// clear the tutorial for keyboard swipe if it is displayed.
    static func focusComposerAndClearTutorialIfNeeded(_ app: XCUIApplication) throws {
        app.textViews["WysiwygComposer"].tap()
        let continueButton = app.buttons["Continue"]
        // If a continue button exists, we are on the keyboard Swipe tutorial.
        if continueButton.exists {
            continueButton.tap()
        }
    }

    /// Type a text and delete some different kind of text selections with Wysiwyg composer inside given app.
    static func testTypingAndDeleting(_ app: XCUIApplication) throws {
        let textView = app.textViews["WysiwygComposer"]
        // Type something into composer.
        textView.typeTextCharByChar("abc🎉🎉👩🏿‍🚀")
        XCTAssertEqual(textView.value as? String, "abc🎉🎉👩🏿‍🚀")

        // Test deleting parts of the text.
        let deleteKey = app.keys["delete"]
        deleteKey.tap()
        XCTAssertEqual(textView.value as? String, "abc🎉🎉")

        let delete3CharString = String(repeating: XCUIKeyboardKey.delete.rawValue, count: 3)
        textView.typeTextCharByChar(delete3CharString)
        XCTAssertEqual(textView.value as? String, "ab")

        // Rewrite some content.
        textView.typeTextCharByChar("cde 🥳 fgh")
        XCTAssertEqual(textView.value as? String, "abcde 🥳 fgh")

        // Double tap results in selecting the last word.
        textView.doubleTap()
        deleteKey.tap()
        XCTAssertEqual(textView.value as? String, "abcde 🥳 ")

        // Triple tap selects the entire line.
        textView.tap(withNumberOfTaps: 3, numberOfTouches: 1)
        deleteKey.tap()
        XCTAssertEqual(textView.value as? String, "")
    }

    /// Type a text and make it bold in Wysiwyg composer inside given app.
    /// A screenshot is saved since string attributes can't be read from this context.
    static func testTypingAndBolding(_ app: XCUIApplication) throws -> XCTAttachment {
        let textView = app.textViews["WysiwygComposer"]
        // Type something into composer.
        textView.typeTextCharByChar("Some bold text")

        textView.doubleTap()
        // We can't detect data being properly reported back to the model but
        // 1s is more than enough for the Rust side to get notified for the selection.
        sleep(1)

        let boldButton = app.buttons[rawIdentifier(.boldButton)]
        boldButton.tap()
        // Bolding doesn't change text and we can't test text attributes from this context.
        XCTAssertEqual(textView.value as? String, "Some bold text")

        // Keep a screenshot of the bolded text.
        let screenshot = textView.screenshot()
        let attachment = XCTAttachment(screenshot: screenshot)
        attachment.lifetime = .keepAlways
        return attachment
    }

    /// Type and send a message with Wysiwyg composer inside given app.
    ///
    /// Expected plain text content is "Some bold text" and
    /// HTML representation is "Some bold <strong>text</strong>"
    static func typeAndSendMessage(_ app: XCUIApplication) throws {
        let textView = app.textViews["WysiwygComposer"]
        // Type something into composer.
        textView.typeTextCharByChar("Some bold text")

        textView.doubleTap()
        // 1s is more than enough for the Rust side to get notified for the selection.
        sleep(1)

        let boldButton = app.buttons[rawIdentifier(.boldButton)]
        boldButton.tap()

        // We can't detect data being properly reported back to the model but
        // 1s is more than enough for the Rust side to get notified for the selection.
        sleep(1)

        let sendButton = app.buttons[rawIdentifier(.sendButton)]
        sendButton.tap()

        let content = app.staticTexts[rawIdentifier(.contentText)]
        let htmlContent = app.staticTexts[rawIdentifier(.htmlContentText)]

        XCTAssertEqual(content.label, "Some bold __text__")
        XCTAssertEqual(htmlContent.label, "Some bold <strong>text</strong>")
    }
    
    static func typingFast(_ app: XCUIApplication) throws {
        let text = "Some long text that I am going to type very fast"
        let textView = app.textViews["WysiwygComposer"]
        textView.tap()
        sleep(1)
        textView.typeText(text)
        let textToVerify = textView.value as? String
        XCTAssert(text == textToVerify)
    }
    
    static func longPressDelete(_ app: XCUIApplication) throws {
        let multilineText =
            """
            test1
            test2
            test3
            test4
            test5
            test6
            test7
            test8
            test9
            test10
            """
        let textView = app.textViews["WysiwygComposer"]
        app.typeTextCharByChar(multilineText)
        XCUIApplication().keys["delete"].press(forDuration: 10.0)
        let resultText = textView.value as? String
        XCTAssert(resultText == "")
    }
}

private extension WysiwygSharedTests {
    static func rawIdentifier(_ id: WysiwygSharedAccessibilityIdentifier) -> String {
        id.rawValue
    }
}

private extension XCUIElement {
    /// Types a text inside the UI element character by character.
    /// This is especially useful to avoid missing some characters on
    /// UI tests running on a rather slow CI.
    ///
    /// - Parameters:
    ///   - text: Text to type in the UI element.
    func typeTextCharByChar(_ text: String) {
        text.forEach { self.typeText(String($0)) }
    }
}
