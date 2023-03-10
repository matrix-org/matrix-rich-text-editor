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
    func testMarkdownFromPlainTextModeIsParsed() throws {
        textView.typeTextCharByChar("text ")
        button(.plainRichButton).tap()
        textView.typeTextCharByChar("__bold__ *italic*")
        assertTextViewContent("text __bold__ *italic*")
        button(.plainRichButton).tap()
        assertTextViewContent("text bold italic")
        assertTreeEquals(
            """
            ├>"text "
            ├>strong
            │ └>"bold"
            ├>" "
            └>em
              └>"italic"
            """
        )
        // Re-toggling restores the markdown.
        button(.plainRichButton).tap()
        assertTextViewContent("text __bold__ *italic*")
    }

    func testPlainTextModePreservesPills() throws {
        // Create a Pill in RTE.
        textView.typeTextCharByChar("@ali")
        button(.aliceButton).tap()
        // Switch to plain text mode and assert Pill exists
        button(.plainRichButton).tap()
        assertMatchingPill("Alice")
        // Write something.
        textView.typeTextCharByChar("hello")
        // Switch back to RTE and assert model.
        button(.plainRichButton).tap()
        assertMatchingPill("Alice")
        assertTreeEquals(
            """
            ├>a "https://matrix.to/#/@alice:matrix.org"
            │ └>"Alice"
            └>" hello"
            """
        )
    }
}
