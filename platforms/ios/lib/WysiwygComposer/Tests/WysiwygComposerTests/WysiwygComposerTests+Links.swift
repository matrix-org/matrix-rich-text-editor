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
        ComposerModelWrapper()
            .assertLinkAction(.createWithText)
    }

    func testCreateLinkAction() {
        ComposerModelWrapper()
            .action { $0.replaceText(newText: "test") }
            .action { $0.select(startUtf16Codeunit: 0, endUtf16Codeunit: 4) }
            .assertLinkAction(.create)
    }

    func testEditLinkAction() {
        let url = "test_url"
        ComposerModelWrapper()
            .action { $0.setLinkWithText(url: url, text: "test", attributes: []) }
            .assertLinkAction(.edit(url: "https://\(url)"))
    }

    func testSetLinkWithText() {
        ComposerModelWrapper()
            .action { $0.setLinkWithText(url: "link", text: "text", attributes: []) }
            .assertTree(
                """

                └>a \"https://link\"
                  └>\"text\"

                """
            )
    }
    
    func testSetLinkWithTextWithIncludedScheme() {
        ComposerModelWrapper()
            .action { $0.setLinkWithText(url: "http://link", text: "text", attributes: []) }
            .assertTree(
                """

                └>a \"http://link\"
                  └>\"text\"

                """
            )
    }
    
    func testSetMailLinkWithText() {
        ComposerModelWrapper()
            .action { $0.setLinkWithText(url: "test@element.io", text: "text", attributes: []) }
            .assertTree(
                """

                └>a \"mailto:test@element.io\"
                  └>\"text\"

                """
            )
    }

    func testSetLink() {
        ComposerModelWrapper()
            .action { $0.replaceText(newText: "text") }
            .action { $0.select(startUtf16Codeunit: 0, endUtf16Codeunit: 4) }
            .action { $0.setLink(url: "link", attributes: []) }
            .assertTree(
                """

                └>a \"https://link\"
                  └>\"text\"

                """
            )
    }

    func testRemoveLinks() {
        ComposerModelWrapper()
            .action { $0.setLinkWithText(url: "link", text: "text", attributes: []) }
            .assertTree(
                """

                └>a \"https://link\"
                  └>\"text\"

                """
            )
            .action { $0.removeLinks() }
            .assertTree(
                """

                └>"text"

                """
            )
    }
}
