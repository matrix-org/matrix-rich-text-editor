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

class WysiwygUITests: XCTestCase {
    private let app = XCUIApplication(bundleIdentifier: "org.matrix.Wysiwyg")

    override func setUpWithError() throws {
        continueAfterFailure = false
        app.launch()
        try focusComposerAndClearTutorialIfNeeded()
    }

    override func tearDownWithError() throws { }

    /// Type a text and delete some different kind of text selections with the composer.
    func testTypingAndDeleting() throws {
        // Type something into composer.
        textView.typeTextCharByChar("abc🎉🎉👩🏿‍🚀")
        assertTextViewContent("abc🎉🎉👩🏿‍🚀")

        // Test deleting parts of the text.
        let deleteKey = app.keys["delete"]
        deleteKey.tap()
        assertTextViewContent("abc🎉🎉")

        let delete3CharString = String(repeating: XCUIKeyboardKey.delete.rawValue, count: 3)
        textView.typeTextCharByChar(delete3CharString)
        assertTextViewContent("ab")

        // Rewrite some content.
        textView.typeTextCharByChar("cde 🥳 fgh")
        assertTextViewContent("abcde 🥳 fgh")

        // Double tap results in selecting the last word.
        textView.doubleTap()
        deleteKey.tap()
        assertTextViewContent("abcde 🥳 ")

        // Triple tap selects the entire line.
        textView.tap(withNumberOfTaps: 3, numberOfTouches: 1)
        deleteKey.tap()
        assertTextViewContent("")
    }

    /// Type a text and make it bold in the composer.
    /// A screenshot is saved since string attributes can't be read from this context.
    func testTypingAndBolding() throws {
        // Type something into composer.
        textView.typeTextCharByChar("Some bold text")

        textView.doubleTap()
        // We can't detect data being properly reported back to the model but
        // 1s is more than enough for the Rust side to get notified for the selection.
        sleep(1)

        button(.boldButton).tap()
        // Bolding doesn't change text and we can't test text attributes from this context.
        assertTextViewContent("Some bold text")

        // Keep a screenshot of the bolded text.
        let screenshot = textView.screenshot()
        let attachment = XCTAttachment(screenshot: screenshot)
        attachment.lifetime = .keepAlways
        add(attachment)
    }

    /// Type and send a message with the composer.
    ///
    /// Expected plain text content is "Some bold text" and
    /// HTML representation is "Some bold <strong>text</strong>"
    func testTypingAndSending() throws {
        // Type something into composer.
        textView.typeTextCharByChar("Some bold text")

        textView.doubleTap()
        // 1s is more than enough for the Rust side to get notified for the selection.
        sleep(1)
        button(.boldButton).tap()
        // We can't detect data being properly reported back to the model but
        // 1s is more than enough for the Rust side to get notified for the selection.
        sleep(1)
        button(.sendButton).tap()

        XCTAssertEqual(staticText(.contentText).label, "Some bold __text__")
        XCTAssertEqual(staticText(.htmlContentText).label, "Some bold <strong>text</strong>")
    }
    
    // Remember to disable hardware keyboard and use only software keyboard for this UITest
    func testTypingFast() throws {
        let text = "Some long text that I am going to type very fast"
        textView.tap()
        sleep(1)
        textView.typeText(text)
        let options = XCTExpectedFailure.Options()
        options.isStrict = false
        XCTExpectFailure("Typing fast might fail on CI", options: options)
        assertTextViewContent(text)
    }
    
    func testLongPressDelete() throws {
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
        app.typeTextCharByChar(multilineText)
        XCUIApplication().keys["delete"].press(forDuration: 10.0)
        assertTextViewContent("")
    }

    func testMinMaxResizing() throws {
        sleep(1)
        XCTAssertEqual(textView.frame.height, WysiwygSharedConstants.composerMinHeight)
        button(.minMaxButton).tap()
        sleep(1)
        XCTAssertEqual(textView.frame.height, WysiwygSharedConstants.composerMaxExtendedHeight)
        button(.minMaxButton).tap()
        sleep(1)
        XCTAssertEqual(textView.frame.height, WysiwygSharedConstants.composerMinHeight)
    }
}

private extension WysiwygUITests {
    /// Returns the text view component of the composer.
    var textView: XCUIElement {
        app.textViews[rawIdentifier(.composerTextView)]
    }

    /// Get the button with given id
    ///
    /// - Parameter id: Accessibility identifier
    /// - Returns: Associated button, if it exists
    func button(_ id: WysiwygSharedAccessibilityIdentifier) -> XCUIElement {
        app.buttons[rawIdentifier(id)]
    }

    /// Get the static text with given id
    ///
    /// - Parameter id: Accessibility identifier
    /// - Returns: Associated static text, if it exists
    func staticText(_ id: WysiwygSharedAccessibilityIdentifier) -> XCUIElement {
        app.staticTexts[rawIdentifier(id)]
    }

    /// Helper for a XCTAssert on the current content of the composer's text view.
    func assertTextViewContent(_ content: @autoclosure () throws -> String,
                               _ message: @autoclosure () -> String = "",
                               file: StaticString = #filePath,
                               line: UInt = #line) {
        XCTAssertEqual(textView.value as? String, try content(), message(), file: file, line: line)
    }

    /// Focus the composer text view inside given app and
    /// clear the tutorial for keyboard swipe if it is displayed.
    func focusComposerAndClearTutorialIfNeeded() throws {
        textView.tap()
        let continueButton = app.buttons["Continue"]
        // If a continue button exists, we are on the keyboard Swipe tutorial.
        if continueButton.exists {
            continueButton.tap()
        }
    }

    /// Get the raw value of an UI element accessibility identifier
    ///
    /// - Parameter id: accessibility identifier of the UI element
    /// - Returns: raw string value
    func rawIdentifier(_ id: WysiwygSharedAccessibilityIdentifier) -> String {
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
