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
    func testIsContentEmpty() throws {
        let viewModel = WysiwygComposerViewModel()
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

        viewModel.replaceText(NSAttributedString(string: ""),
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

        viewModel.replaceText(viewModel.content.attributed,
                              range: .init(location: 0, length: viewModel.content.attributed.length),
                              replacementText: "")

        wait(for: [expectTrue], timeout: 2.0)
        cancellableTrue.cancel()
    }

    func testKeyboardAutocompleteWhitespaceMissSelection() throws {
        let viewModel = WysiwygComposerViewModel()
        // Selection that will get block by the view model to avoid wrong OS inputs.
        let missSelectionRange = NSRange(location: 8, length: 0)

        viewModel.replaceText(NSAttributedString(string: ""),
                              range: .zero,
                              replacementText: "Autocomp")
        // Simulate the first input from a keyboard autocomplete.
        viewModel.replaceText(NSAttributedString(string: "Autocomp"),
                              range: NSRange(location: 0, length: 8),
                              replacementText: "Autocomplete")
        // Miss selection range is locked
        viewModel.select(text: NSAttributedString(string: "Autocomp"),
                         range: missSelectionRange)
        XCTAssertNotEqual(viewModel.content.attributedSelection,
                          missSelectionRange)
        // Simulate the second input from a keyboard autocomplete (e.g. with wrong location).
        viewModel.replaceText(NSAttributedString(string: "Autocomp"),
                              range: missSelectionRange,
                              replacementText: " ")
        // Content is correct and the cursor is at the expected location
        XCTAssertEqual(viewModel.content.plainText,
                       "Autocomplete ")
        XCTAssertEqual(viewModel.content.attributedSelection,
                       NSRange(location: 13, length: 0))
        // Previous miss selection range is now selectable
        viewModel.select(text: NSAttributedString(string: "Autocomplete "),
                         range: missSelectionRange)
        XCTAssertEqual(viewModel.content.attributedSelection,
                       missSelectionRange)
    }

    func testKeyboardReplaceWithMissSelection() throws {
        let viewModel = WysiwygComposerViewModel()
        // Selection that will get block by the view model to avoid wrong OS inputs.
        let missSelectionRange = NSRange(location: 13, length: 0)

        viewModel.replaceText(NSAttributedString(string: ""),
                              range: .zero,
                              replacementText: "Some incorect text")
        // Simulate the first input from a keyboard replace.
        viewModel.replaceText(NSAttributedString(string: "Some incorect text"),
                              range: NSRange(location: 5, length: 8),
                              replacementText: "incorrect")
        // Miss selection range is locked
        viewModel.select(text: NSAttributedString(string: "Autocomp"),
                         range: missSelectionRange)
        XCTAssertNotEqual(viewModel.content.attributedSelection,
                          missSelectionRange)
        // Simulate the second input from a keyboard replace (e.g. with wrong location).
        viewModel.replaceText(NSAttributedString(string: "Some incorect text"),
                              range: missSelectionRange,
                              replacementText: "")
        // Content is correct and the cursor is at the expected location
        XCTAssertEqual(viewModel.content.plainText,
                       "Some incorrect text")
        XCTAssertEqual(viewModel.content.attributedSelection,
                       NSRange(location: 14, length: 0))
        // Previous miss selection range is now selectable
        viewModel.select(text: NSAttributedString(string: "Some incorrect text"),
                         range: missSelectionRange)
        XCTAssertEqual(viewModel.content.attributedSelection,
                       missSelectionRange)
    }

    func testStandardTextInput() throws {
        // FIXME: fix unselectable range issue on standard text input and re-enable this
        let viewModel = WysiwygComposerViewModel()
        viewModel.replaceText(NSAttributedString(string: ""),
                              range: .zero,
                              replacementText: "A")
        viewModel.select(text: NSAttributedString(string: "A"),
                         range: .zero)
        XCTAssertEqual(viewModel.content.attributedSelection, .zero)
    }
}
