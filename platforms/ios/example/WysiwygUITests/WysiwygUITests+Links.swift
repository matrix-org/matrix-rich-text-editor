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
    func testCreateLinkWithTextEditAndRemove() {
        // Create with text
        button(.linkButton).tap()
        XCTAssertTrue(textField(.linkUrlTextField).exists)
        XCTAssertTrue(textField(.linkTextTextField).exists)
        textField(.linkUrlTextField).typeTextCharByChar("url")
        textField(.linkTextTextField).tap()
        textField(.linkTextTextField).typeTextCharByChar("text")
        app.buttons["Ok"].tap()
        assertTreeEquals(
            """
            └>a "https://url"
              └>"text"
            """
        )

        // Edit
        button(.linkButton).tap()
        XCTAssertTrue(textField(.linkUrlTextField).exists)
        XCTAssertTrue(textField(.linkTextTextField).exists)
        textField(.linkUrlTextField).doubleTap()
        textField(.linkUrlTextField).typeTextCharByChar("new_url")
        app.buttons["Ok"].tap()
        assertTreeEquals(
            """
            └>a "https://new_url"
              └>"text"
            """
        )

        // Remove
        button(.linkButton).tap()
        XCTAssertFalse(textField(.linkTextTextField).exists)
        app.buttons["Remove"].tap()
        assertTreeEquals(
            """
            └>"text"
            """
        )
    }

    func testCreateLinkFromSelection() {
        textView.typeTextCharByChar("text")
        assertTreeEquals(
            """
            └>"text"
            """
        )

        textView.doubleTap()
        button(.linkButton).tap()
        XCTAssertFalse(textField(.linkTextTextField).exists)
        textField(.linkUrlTextField).typeTextCharByChar("url")
        app.buttons["Ok"].tap()
        assertTreeEquals(
            """
            └>a "https://url"
              └>"text"
            """
        )
    }
}
