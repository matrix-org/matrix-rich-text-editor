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

@testable import WysiwygComposer
import XCTest

extension WysiwygComposerTests {
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
}
