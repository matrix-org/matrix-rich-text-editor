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

import XCTest
import Combine
@testable import WysiwygComposer

final class WysiwygComposerViewModelTests: XCTestCase {
    private let viewModel = WysiwygComposerViewModel()

    override func setUpWithError() throws {
        viewModel.clearContent()
    }

    func testIsContentEmpty() throws {
        XCTAssertTrue(viewModel.isContentEmpty)

        let expectFalse = self.expectation(description: "Await isContentEmpty false")
        let cancellableFalse = viewModel.$isContentEmpty
        // Ignore on subscribe publish.
            .dropFirst()
            .removeDuplicates()
            .sink(receiveValue: { isEmpty in
                XCTAssertFalse(isEmpty)
                expectFalse.fulfill()
            })

        _ = viewModel.replaceText(NSAttributedString(string: ""),
                                  range: .zero,
                                  replacementText: "Test")

        wait(for: [expectFalse], timeout: 2.0)
        cancellableFalse.cancel()

        let expectTrue = self.expectation(description: "Await isContentEmpty true")
        let cancellableTrue = viewModel.$isContentEmpty
        // Ignore on subscribe publish.
            .dropFirst()
            .removeDuplicates()
            .sink(receiveValue: { isEmpty in
                XCTAssertTrue(isEmpty)
                expectTrue.fulfill()
            })

        _ = viewModel.replaceText(viewModel.content.attributed,
                                  range: .init(location: 0, length: viewModel.content.attributed.length),
                                  replacementText: "")

        wait(for: [expectTrue], timeout: 2.0)
        cancellableTrue.cancel()
    }

    func testSimpleTextInputIsAccepted() throws {
        let shouldChange = viewModel.replaceText(NSAttributedString(string: ""),
                                                 range: .zero,
                                                 replacementText: "A")
        XCTAssertTrue(shouldChange)
    }

    func testNewlineIsNotAccepted() throws {
        let shouldChange = viewModel.replaceText(NSAttributedString(string: ""),
                                                 range: .zero,
                                                 replacementText: "\n")
        XCTAssertFalse(shouldChange)
    }

    func testReconciliateTextView() {
        let textView = UITextView()
        let initialText = NSAttributedString(string: "")
        textView.attributedText = initialText
        _ = viewModel.replaceText(initialText,
                                  range: .zero,
                                  replacementText: "A")
        textView.attributedText = NSAttributedString(string: "AA")
        XCTAssertEqual(textView.text, "AA")
        XCTAssertEqual(textView.selectedRange, NSRange(location: 2, length: 0))
        viewModel.didUpdateText(textView: textView)
        XCTAssertEqual(textView.text, "A")
        XCTAssertEqual(textView.selectedRange, NSRange(location: 1, length: 0))
    }
}
