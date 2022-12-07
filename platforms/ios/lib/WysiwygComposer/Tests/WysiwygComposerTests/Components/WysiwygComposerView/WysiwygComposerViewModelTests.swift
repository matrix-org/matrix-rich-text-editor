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
    }

    func testIsContentEmpty() throws {
        XCTAssertTrue(viewModel.isContentEmpty)

        let expectFalse = expectation(description: "Await isContentEmpty false")
        let cancellableFalse = viewModel.$isContentEmpty
            // Ignore on subscribe publish.
            .removeDuplicates()
            .dropFirst()
            .sink(receiveValue: { isEmpty in
                XCTAssertFalse(isEmpty)
                expectFalse.fulfill()
            })

        _ = viewModel.replaceText(range: .zero,
                                  replacementText: "Test")
        viewModel.textView.attributedText = viewModel.attributedContent.text

        wait(for: [expectFalse], timeout: 2.0)
        cancellableFalse.cancel()

        let expectTrue = expectation(description: "Await isContentEmpty true")
        let cancellableTrue = viewModel.$isContentEmpty
            // Ignore on subscribe publish.
            .removeDuplicates()
            .dropFirst()
            .sink(receiveValue: { isEmpty in
                XCTAssertTrue(isEmpty)
                expectTrue.fulfill()
            })

        _ = viewModel.replaceText(range: .init(location: 0, length: viewModel.attributedContent.text.length),
                                  replacementText: "")
        viewModel.textView.attributedText = viewModel.attributedContent.text

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

    func testReconciliateModel() {
        _ = viewModel.replaceText(range: .zero,
                                  replacementText: "wa")
        XCTAssertEqual(viewModel.attributedContent.text.string, "wa")
        XCTAssertEqual(viewModel.attributedContent.selection, NSRange(location: 2, length: 0))
        reconciliate(to: "わ", selectedRange: NSRange(location: 1, length: 0))
        XCTAssertEqual(viewModel.attributedContent.text.string, "わ")
        XCTAssertEqual(viewModel.attributedContent.selection, NSRange(location: 1, length: 0))
    }

    func testReconciliateRestoresSelection() {
        _ = viewModel.replaceText(range: .zero, replacementText: "I\'m")
        XCTAssertEqual(viewModel.attributedContent.selection, NSRange(location: 3, length: 0))
        reconciliate(to: "I’m", selectedRange: NSRange(location: 3, length: 0))
        XCTAssertEqual(viewModel.attributedContent.selection, NSRange(location: 3, length: 0))

        viewModel.clearContent()

        _ = viewModel.replaceText(range: .zero, replacementText: "Some text")
        viewModel.select(range: .zero)
        XCTAssertEqual(viewModel.attributedContent.selection, .zero)
        reconciliate(to: "Some test", selectedRange: .zero)
        XCTAssertEqual(viewModel.attributedContent.selection, .zero)
    }

    func testReconciliateRestoresFromModel() {
        _ = viewModel.replaceText(range: .zero, replacementText: "Some text")
        viewModel.textView.attributedText = NSAttributedString(string: "Some text")
        reconciliate(to: "Home test", selectedRange: .zero)
        XCTAssertEqual(viewModel.textView.text, "Some text")
    }

    func testPlainTextMode() {
        _ = viewModel.replaceText(range: .zero,
                                  replacementText: "Some bold text")
        viewModel.textView.attributedText = NSAttributedString(string: "Some bold text")
        viewModel.select(range: .init(location: 10, length: 4))
        viewModel.apply(.bold)

        XCTAssertEqual(viewModel.content.html, "Some bold <strong>text</strong>")

        viewModel.plainTextMode = true
        XCTAssertEqual(viewModel.content.markdown, "Some bold __text__")
        XCTAssertEqual(viewModel.content.html, "Some bold <strong>text</strong>")

        viewModel.plainTextMode = false
        XCTAssertEqual(viewModel.content.html, "Some bold <strong>text</strong>")
    }
    
    func testReplaceTextAfterLinkIsNotAccepted() {
        viewModel.applyLinkOperation(.createLink(urlString: "https://element.io", text: "test"))
        let result = viewModel.replaceText(range: .init(location: 4, length: 0), replacementText: "abc")
        XCTAssertFalse(result)
        XCTAssertEqual(viewModel.content.html, "<a href=\"https://element.io\">test</a>abc")
        XCTAssertTrue(viewModel.textView.attributedText.isEqual(to: viewModel.attributedContent.text))
    }
    
    func testReplaceTextPartiallyInsideAndAfterLinkIsNotAccepted() {
        viewModel.applyLinkOperation(.createLink(urlString: "https://element.io", text: "test"))
        let result = viewModel.replaceText(range: .init(location: 3, length: 1), replacementText: "abc")
        XCTAssertFalse(result)
        XCTAssertEqual(viewModel.content.html, "<a href=\"https://element.io\">tes</a>abc")
        XCTAssertTrue(viewModel.textView.attributedText.isEqual(to: viewModel.attributedContent.text))
    }
    
    func testReplaceTextInsideLinkIsAccepted() {
        viewModel.applyLinkOperation(.createLink(urlString: "https://element.io", text: "test"))
        let result = viewModel.replaceText(range: .init(location: 2, length: 0), replacementText: "abc")
        XCTAssertTrue(result)
        XCTAssertEqual(viewModel.content.html, "<a href=\"https://element.io\">teabcst</a>")
    }
}

private extension WysiwygComposerViewModelTests {
    /// Fakes a trigger of the reconciliate mechanism of the view model.
    ///
    /// - Parameters:
    ///   - newText: New text to apply.
    ///   - selectedRange: Simulated selection in the text view.
    func reconciliate(to newText: String, selectedRange: NSRange) {
        viewModel.textView.attributedText = NSAttributedString(string: newText)
        // Set selection where we want it, as setting the content automatically moves cursor to the end.
        viewModel.textView.selectedRange = selectedRange
        viewModel.didUpdateText()
    }
}
