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
    func testAtMention() throws {
        textView.typeTextCharByChar("@ali")
        XCTAssertTrue(button(.aliceButton).exists)
        button(.aliceButton).tap()
        assertMatchingPill("Alice")
        // Mention is replaced by a pill view, so there
        // is only the space after it in the field.
        assertTextViewContent("￼\u{00A0}")
        assertTreeEquals(
            """
            ├>a "https://matrix.to/#/@alice:matrix.org"
            │ └>"Alice"
            └>" "
            """
        )
    }

    func testHashMention() throws {
        textView.typeTextCharByChar("#roo")
        XCTAssertTrue(button(.room1Button).exists)
        button(.room1Button).tap()
        // FIXME: room links are not considered valid links through parsing, so no mention is displayed atm
        // assertTextViewContent("￼ ")
        assertTreeEquals(
            """
            ├>a "https://matrix.to/#/#room1:matrix.org"
            │ └>"Room 1"
            └>" "
            """
        )
    }

    func testCommand() throws {
        textView.typeTextCharByChar("/inv")
        XCTAssertTrue(button(.inviteCommandButton).exists)
        button(.inviteCommandButton).tap()
        assertTextViewContent("/invite\u{00A0}")
        assertTreeEquals(
            """
            └>"/invite "
            """
        )
    }
}
