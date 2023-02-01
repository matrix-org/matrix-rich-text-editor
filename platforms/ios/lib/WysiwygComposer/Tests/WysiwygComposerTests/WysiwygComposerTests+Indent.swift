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
        newComposerModel()
            .action { $0.setContentFromHtml(html: Constants.sampleListHtml) }
            // Select somewhere on item 2
            .action { $0.select(startUtf16Codeunit: 9, endUtf16Codeunit: 9) }
            .action { $0.indent() }
            .execute { XCTAssertTrue($0.actionStates()[.indent] == .disabled) }
            // Select somewhere on item 3
            .action { $0.select(startUtf16Codeunit: 18, endUtf16Codeunit: 18) }
            .action { $0.indent() }
            .action { $0.indent() }
            .execute { XCTAssertTrue($0.actionStates()[.indent] == .disabled) }
            .assertHtml(Constants.indentedSampleListHtml)
    }

    func testUnindent() {
        newComposerModel()
            .action { $0.setContentFromHtml(html: Constants.indentedSampleListHtml) }
            // Select somewhere on item 3
            .action { $0.select(startUtf16Codeunit: 18, endUtf16Codeunit: 18) }
            .action { $0.unindent() }
            .action { $0.unindent() }
            .execute { XCTAssertTrue($0.actionStates()[.unindent] == .disabled) }
            // Select somewhere on item 2
            .action { $0.select(startUtf16Codeunit: 9, endUtf16Codeunit: 9) }
            .action { $0.unindent() }
            .execute { XCTAssertTrue($0.actionStates()[.unindent] == .disabled) }
            .assertHtml(Constants.sampleListHtml)
    }
}
