//
// Copyright 2023 The Matrix.org Foundation C.I.C
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

extension WysiwygUITests {
    func disable_testRichTextModeAutocorrection() throws {
        textView.typeTextCharByChar("/")
        XCTAssertFalse(image(.autocorrectionIndicator).exists)
        textView.typeText(XCUIKeyboardKey.delete.rawValue)
        XCTAssertTrue(image(.autocorrectionIndicator).exists)
        textView.typeTextCharByChar("/join")
        XCTAssertFalse(image(.autocorrectionIndicator).exists)
        // Send message
        button(.sendButton).tap()
        XCTAssertTrue(image(.autocorrectionIndicator).exists)
    }

    func disable_testPlainTextModeAutocorrection() throws {
        waitForButtonToExistAndTap(.plainRichButton)
        textView.typeTextCharByChar("/")
        XCTAssertFalse(image(.autocorrectionIndicator).exists)
        textView.typeText(XCUIKeyboardKey.delete.rawValue)
        XCTAssertTrue(image(.autocorrectionIndicator).exists)
        textView.typeTextCharByChar("/join")
        XCTAssertFalse(image(.autocorrectionIndicator).exists)
        // Send message
        button(.sendButton).tap()
        XCTAssertTrue(image(.autocorrectionIndicator).exists)
    }

    func testRichTextModeNonLeadingCommand() throws {
        textView.typeTextCharByChar("text /not_a_command")
        XCTAssertTrue(image(.autocorrectionIndicator).exists)
    }

    func testPlainTextModeNonLeadingCommand() throws {
        waitForButtonToExistAndTap(.plainRichButton)
        textView.typeTextCharByChar("text /not_a_command")
        XCTAssertTrue(image(.autocorrectionIndicator).exists)
    }
}
