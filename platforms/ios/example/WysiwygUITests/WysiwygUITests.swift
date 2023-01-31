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
    internal let app = XCUIApplication(bundleIdentifier: "org.matrix.Wysiwyg")

    override func setUpWithError() throws {
        continueAfterFailure = false
        app.launch()
        try focusComposerAndClearTutorialIfNeeded()
    }

    override func tearDownWithError() throws { }

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

internal extension WysiwygUITests {
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
    
    func textField(_ id: WysiwygSharedAccessibilityIdentifier) -> XCUIElement {
        app.textFields[id.rawValue]
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
        guard var text = textView.value as? String else {
            XCTFail("Unable to retrieve text view content")
            return
        }
        // Remove occurences of ZWSP to avoid issues with expected content.
        text = text.replacingOccurrences(of: "\u{200B}", with: "")
        XCTAssertEqual(text, try content(), message(), file: file, line: line)
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
    
    /// Shows or updates the current tree content of the text view and checks if is equal to provided content
    ///
    /// - Parameter content: the tree content to assert, must be provided without newlines at the start and at the end.
    func assertTreeEquals(_ content: String) {
        sleep(1)
        XCTAssertEqual(staticText(.treeText).label, "\n\(content)\n")
    }
}

internal extension XCUIElement {
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
