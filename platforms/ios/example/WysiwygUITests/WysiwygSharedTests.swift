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

/// Defines tests that can be shared between the SwiftUI and the UIKit example implementations.
final class WysiwygSharedTests {
    private init() { }

    static func testTypingAndDeleting(_ app: XCUIApplication) throws {
        let textView = app.textViews["WysiwygComposer"]
        // Select text view and type something.
        textView.tap()
        textView.typeText("abc🎉🎉👩🏿‍🚀")
        XCTAssertEqual(textView.value as? String, "abc🎉🎉👩🏿‍🚀")

        // Test deleting parts of the text.
        let deleteKey = app.keys["delete"]
        deleteKey.tap()
        XCTAssertEqual(textView.value as? String, "abc🎉🎉")

        let delete3CharString = String(repeating: XCUIKeyboardKey.delete.rawValue, count: 3)
        textView.typeText(delete3CharString)
        XCTAssertEqual(textView.value as? String, "ab")

        // Rewrite some content.
        textView.typeText("cde 🥳 fgh")
        XCTAssertEqual(textView.value as? String, "abcde 🥳 fgh")

        // Double tap results in selecting the last word.
        textView.doubleTap()
        deleteKey.tap()
        XCTAssertEqual(textView.value as? String, "abcde 🥳 ")

        // Triple tap selects the entire line.
        textView.tap(withNumberOfTaps: 3, numberOfTouches: 1)
        deleteKey.tap()
        XCTAssertEqual(textView.value as? String, "")
    }

    static func testTypingAndBolding(_ app: XCUIApplication) throws -> XCTAttachment {
        let textView = app.textViews["WysiwygComposer"]
        // Select text view and type something.
        textView.tap()
        textView.typeText("Some bold text")

        textView.doubleTap()
        // We can't detect data being properly reported back to the model but
        // 1s is more than enough for the Rust side to get notified for the selection.
        sleep(1)

        let boldButton = app.buttons["WysiwygBoldButton"]
        boldButton.tap()
        // Bolding doesn't change text and we can't test text attributes from this context.
        XCTAssertEqual(textView.value as? String, "Some bold text")

        // Keep a screenshot of the bolded text.
        let screenshot = textView.screenshot()
        let attachment = XCTAttachment(screenshot: screenshot)
        attachment.lifetime = .keepAlways
        return attachment
    }
}
