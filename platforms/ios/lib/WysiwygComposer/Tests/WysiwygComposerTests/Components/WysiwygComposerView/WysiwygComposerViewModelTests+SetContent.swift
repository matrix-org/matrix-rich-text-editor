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

import Combine
@testable import WysiwygComposer
import XCTest

private enum Constants {
    static let sampleHtml = "some <strong>bold</strong> text"
    static let sampleMarkdown = "some __bold__ text"
    static let samplePlainText = "some bold text"
    static let sampleHtml2 = "<ol><li><strong>A</strong></li><li><em>B</em></li></ol>"
    static let sampleMarkdown2 = "1. __A__\n2. *B*"
}

extension WysiwygComposerViewModelTests {
    func testSetHtmlContent() throws {
        viewModel.setHtmlContent(Constants.sampleHtml)
        XCTAssertEqual(viewModel.content.html, Constants.sampleHtml)
        XCTAssertEqual(viewModel.content.markdown, Constants.sampleMarkdown)

        viewModel.setHtmlContent(Constants.sampleHtml2)
        XCTAssertEqual(viewModel.content.html, Constants.sampleHtml2)
        XCTAssertEqual(viewModel.content.markdown, Constants.sampleMarkdown2)
    }

    func testSetMarkdownContent() throws {
        viewModel.setMarkdownContent(Constants.sampleMarkdown)
        XCTAssertEqual(viewModel.content.html, Constants.sampleHtml)
        XCTAssertEqual(viewModel.content.markdown, Constants.sampleMarkdown)

        viewModel.setMarkdownContent(Constants.sampleMarkdown2)
        XCTAssertEqual(viewModel.content.html, Constants.sampleHtml2)
        XCTAssertEqual(viewModel.content.markdown, Constants.sampleMarkdown2)
    }

    func testSetHtmlContentTriggersPublish() {
        let expectation = expectAttributedContentPublish(Constants.samplePlainText)
        viewModel.setHtmlContent(Constants.sampleHtml)
        waitExpectation(expectation: expectation, timeout: 2.0)
    }

    func testSetMarkdownContentTriggersPublish() {
        let expectation = expectAttributedContentPublish(Constants.samplePlainText)
        viewModel.setMarkdownContent(Constants.sampleMarkdown)
        waitExpectation(expectation: expectation, timeout: 2.0)
    }
}

private extension WysiwygComposerViewModelTests {
    /// Create an expectation for an attributed content to be published by the view model.
    ///
    /// - Parameters:
    ///   - expectedPlainText: Expected plain text.
    ///   - description: Description for expectation.
    /// - Returns: Expectation to be fulfilled. Can be used with `waitExpectation`.
    /// - Note: the plain text is asserted, as its way easier to build than attributed string.
    func expectAttributedContentPublish(_ expectedPlainText: String,
                                        description: String = "Await attributed content") -> WysiwygTestExpectation {
        let expectAttributedContent = expectation(description: description)
        let cancellable = viewModel.$attributedContent
            // Ignore on subscribe publish.
            .removeDuplicates(by: {
                $0.text == $1.text
            })
            .dropFirst()
            .sink(receiveValue: { attributedContent in
                XCTAssertEqual(
                    attributedContent.plainText,
                    expectedPlainText
                )
                expectAttributedContent.fulfill()
            })
        return WysiwygTestExpectation(value: expectAttributedContent, cancellable: cancellable)
    }
}
