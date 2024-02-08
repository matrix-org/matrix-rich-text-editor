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

extension WysiwygComposerViewModelTests {
    func testAutocorrectionIsDisabled() throws {
        mockTrailingTyping("/")
        assertAutocorrectDisabled()

        mockTrailingTyping("join")
        assertAutocorrectDisabled()

        mockTrailingTyping(" #some_room:matrix.org")
        assertAutocorrectDisabled()
    }

    func testAutocorrectionIsEnabled() throws {
        mockTrailingTyping("Just some text")
        assertAutoCorrectEnabled()

        mockTrailingTyping(" /not_a_command")
        assertAutoCorrectEnabled()
    }

    func testDoubleSlashKeepAutocorrectionEnabled() throws {
        mockTrailingTyping("//")
        assertAutoCorrectEnabled()
    }

    func testAutocorrectionIsReEnabled() throws {
        mockTrailingTyping("/")
        assertAutocorrectDisabled()

        mockTrailingBackspace()
        assertAutoCorrectEnabled()

        mockTrailingTyping("/join")
        assertAutocorrectDisabled()

        for _ in 0...4 {
            mockTrailingBackspace()
        }
        assertAutoCorrectEnabled()
    }

    func testAutocorrectionAfterSetHtmlContent() {
        viewModel.setHtmlContent("/join #some_room:matrix.org")
        assertAutocorrectDisabled()

        viewModel.setHtmlContent("<strong>some text</strong>")
        assertAutoCorrectEnabled()
    }

    // Note: disable for now as this is broken by escaping the slash character
    // it could be fixed in `toggleAutocorrectionIfNeeded` text view function
    // but it would have a performance impact
//    func testAutocorrectionAfterSetHtmlContentInPlainTextMode() {
//        viewModel.plainTextMode = true
//
//        viewModel.setHtmlContent("/join #some_room:matrix.org")
//        assertAutocorrectDisabled()
//
//        viewModel.setHtmlContent("<strong>some text</strong>")
//        assertAutoCorrectEnabled()
//    }

    func testAutocorrectionAfterSetMarkdownContent() {
        viewModel.setMarkdownContent("/join #some_room:matrix.org")
        assertAutocorrectDisabled()

        viewModel.setMarkdownContent("__some text__")
        assertAutoCorrectEnabled()
    }

    // Note: disable for now as this is broken by escaping the slash character
    // it could be fixed in `toggleAutocorrectionIfNeeded` text view function
    // but it would have a performance impact
//    func testAutocorrectionAfterSetMarkdownContentInPlainTextMode() {
//        viewModel.plainTextMode = true
//
//        viewModel.setMarkdownContent("/join #some_room:matrix.org")
//        assertAutocorrectDisabled()
//
//        viewModel.setMarkdownContent("__some text__")
//        assertAutoCorrectEnabled()
//    }
}

private extension WysiwygComposerViewModelTests {
    func assertAutoCorrectEnabled() {
        XCTAssertEqual(viewModel.textView?.autocorrectionType, .yes)
    }

    func assertAutocorrectDisabled() {
        XCTAssertEqual(viewModel.textView?.autocorrectionType, .no)
    }
}
