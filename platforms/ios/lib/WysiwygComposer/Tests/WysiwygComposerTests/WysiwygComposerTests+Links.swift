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
        newComposerModel()
            .assertLinkAction(.createWithText)
    }

    func testCreateLinkAction() {
        newComposerModel()
            .action { try $0.replaceText(newText: "test") }
            .action { try $0.select(startUtf16Codeunit: 0, endUtf16Codeunit: 4) }
            .assertLinkAction(.create)
    }

    func testEditLinkAction() {
        let link = "test_url"
        newComposerModel()
            .action { try $0.setLinkWithText(link: link, text: "test") }
            .assertLinkAction(.edit(link: "https://\(link)"))
    }

    func testSetLinkWithText() {
        newComposerModel()
            .action { try $0.setLinkWithText(link: "link", text: "text") }
            .assertTree(
                """

                └>a \"https://link\"
                  └>\"text\"

                """
            )
    }
    
    func testSetLinkWithTextWithIncludedScheme() {
        newComposerModel()
            .action { try $0.setLinkWithText(link: "http://link", text: "text") }
            .assertTree(
                """

                └>a \"http://link\"
                  └>\"text\"

                """
            )
    }
    
    func testSetMailLinkWithText() {
        newComposerModel()
            .action { try $0.setLinkWithText(link: "test@element.io", text: "text") }
            .assertTree(
                """

                └>a \"mailto:test@element.io\"
                  └>\"text\"

                """
            )
    }

    func testSetLink() {
        newComposerModel()
            .action { try $0.replaceText(newText: "text") }
            .action { try $0.select(startUtf16Codeunit: 0, endUtf16Codeunit: 4) }
            .action { try $0.setLink(link: "link") }
            .assertTree(
                """

                └>a \"https://link\"
                  └>\"text\"

                """
            )
    }

    func testRemoveLinks() {
        newComposerModel()
            .action { try $0.setLinkWithText(link: "link", text: "text") }
            .assertTree(
                """

                └>a \"https://link\"
                  └>\"text\"

                """
            )
            .action { try $0.removeLinks() }
            .assertTree(
                """

                └>"text"

                """
            )
    }
}
