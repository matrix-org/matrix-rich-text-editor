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

private enum Constants {
    /// A list with three items.
    static let sampleListHtml = "<ol><li>Item 1</li><li>Item 2</li><li>Item 3</li></ol>"
    /// A list with three items. Second item is indented, Third item is indented twice.
    static let indentedSampleListHtml = """
    <ol><li><p>Item 1</p>\
    <ol><li><p>Item 2</p>\
    <ol><li>Item 3</li></ol></li></ol></li></ol>
    """
}

extension WysiwygComposerTests {
    func testIndent() {
        let composer = newComposerModel()
        _ = composer.setContentFromHtml(html: Constants.sampleListHtml)
        // Select somewhere on item 2
        _ = composer.select(startUtf16Codeunit: 9, endUtf16Codeunit: 9)
        _ = composer.indent()
        XCTAssertTrue(composer.actionStates()[.indent] == .disabled)
        // Select somewhere on item 3
        _ = composer.select(startUtf16Codeunit: 18, endUtf16Codeunit: 18)
        _ = composer.indent()
        _ = composer.indent()
        XCTAssertTrue(composer.actionStates()[.indent] == .disabled)
        XCTAssertEqual(
            composer.getContentAsHtml(),
            Constants.indentedSampleListHtml
        )
    }

    func testUnIndent() {
        let composer = newComposerModel()
        _ = composer.setContentFromHtml(html: Constants.indentedSampleListHtml)
        // Select somewhere on item 3
        _ = composer.select(startUtf16Codeunit: 18, endUtf16Codeunit: 18)
        _ = composer.unIndent()
        _ = composer.unIndent()
        XCTAssertTrue(composer.actionStates()[.unIndent] == .disabled)
        // Select somewhere on item 2
        _ = composer.select(startUtf16Codeunit: 9, endUtf16Codeunit: 9)
        _ = composer.unIndent()
        XCTAssertTrue(composer.actionStates()[.unIndent] == .disabled)
        XCTAssertEqual(
            composer.getContentAsHtml(),
            Constants.sampleListHtml
        )
    }
}
