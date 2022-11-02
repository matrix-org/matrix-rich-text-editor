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

import Combine
@testable import WysiwygComposer
import XCTest

final class WysiwygComposerViewModelTests: XCTestCase {
    private let viewModel = WysiwygComposerViewModel()

    override func setUpWithError() throws {
        viewModel.clearContent()
        viewModel.textView = PlaceholdableTextView()
    }

    func testIsContentEmpty() throws {
        XCTAssertTrue(viewModel.isContentEmpty)

        let expectFalse = expectation(description: "Await isContentEmpty false")
        let cancellableFalse = viewModel.$isContentEmpty
            // Ignore on subscribe publish.
            .dropFirst()
            .removeDuplicates()
            .sink(receiveValue: { isEmpty in
                XCTAssertFalse(isEmpty)
                expectFalse.fulfill()
            })

        _ = viewModel.replaceText(range: .zero,
                                  replacementText: "Test")

        wait(for: [expectFalse], timeout: 2.0)
        cancellableFalse.cancel()

        let expectTrue = expectation(description: "Await isContentEmpty true")
        let cancellableTrue = viewModel.$isContentEmpty
            // Ignore on subscribe publish.
            .dropFirst()
            .removeDuplicates()
            .sink(receiveValue: { isEmpty in
                XCTAssertTrue(isEmpty)
                expectTrue.fulfill()
            })

        viewModel.textView?.attributedText = viewModel.content.attributed
        _ = viewModel.replaceText(range: .init(location: 0, length: viewModel.content.attributed.length),
                                  replacementText: "")

        wait(for: [expectTrue], timeout: 2.0)
        cancellableTrue.cancel()
    }

    func testSimpleTextInputIsAccepted() throws {
        let shouldChange = viewModel.replaceText(range: .zero,
                                                 replacementText: "A")
        XCTAssertTrue(shouldChange)
    }

    func testNewlineIsNotAccepted() throws {
        let shouldChange = viewModel.replaceText(range: .zero,
                                                 replacementText: "\n")
        XCTAssertFalse(shouldChange)
    }

    func testReconciliateTextView() {
        _ = viewModel.replaceText(range: .zero,
                                  replacementText: "A")
        viewModel.textView?.attributedText = NSAttributedString(string: "AA")
        XCTAssertEqual(viewModel.textView?.text, "AA")
        XCTAssertEqual(viewModel.textView?.selectedRange, NSRange(location: 2, length: 0))
        viewModel.didUpdateText()
        XCTAssertEqual(viewModel.textView?.text, "A")
        XCTAssertEqual(viewModel.textView?.selectedRange, NSRange(location: 1, length: 0))
    }

    func testPlainTextMode() {
        _ = viewModel.replaceText(range: .zero,
                                  replacementText: "Some bold text")
        viewModel.textView?.attributedText = NSAttributedString(string: "Some bold text")
        viewModel.select(range: .init(location: 10, length: 4))
        viewModel.apply(.bold)

        XCTAssertEqual(viewModel.content.html, "Some bold <strong>text</strong>")

        viewModel.plainTextMode = true

        XCTExpectFailure("Plain text should contain the appropriate Markdown")
        XCTAssertEqual(viewModel.plainTextModeContent.plainText, "Some bold **text**")
        XCTExpectFailure("HTML should be re-generated from Markdown input upon sending")
        XCTAssertEqual(viewModel.plainTextModeContent.html, "Some bold <strong>text</strong>")

        viewModel.plainTextMode = false

        XCTExpectFailure("Switching back to WYSIWYG should restore the HTML")
        XCTAssertEqual(viewModel.content.html, "Some bold <strong>text</strong>")
    }
}
