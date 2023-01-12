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
    static let resultHtml = "<pre>​Some code\n\tmore code</pre><br />"
    static let resultTree =
        """

        ├>pre
        │ ├>~
        │ └>"Some code\n\tmore code"
        └>br

        """
}

extension WysiwygComposerTests {
    func testCodeBlocksFromEmptyComposer() {
        let composer = newComposerModel()
        _ = composer.codeBlock()
        _ = composer.replaceText(newText: "Some code")
        _ = composer.enter()
        _ = composer.replaceText(newText: "\t")
        _ = composer.replaceText(newText: "more code")
        _ = composer.enter()
        _ = composer.enter()
        XCTAssertEqual(composer.getContentAsHtml(), Constants.resultHtml)
        XCTAssertEqual(composer.toTree(), Constants.resultTree)
    }

    func testCodeBlocksWithMultilineInput() {
        let composer = newComposerModel()
        _ = composer.codeBlock()
        _ = composer.replaceText(newText: "Some code\n\tmore code")
        _ = composer.enter()
        _ = composer.enter()
        XCTAssertEqual(composer.getContentAsHtml(), Constants.resultHtml)
        XCTAssertEqual(composer.toTree(), Constants.resultTree)
    }

    func testCodeBlocksFromContent() {
        let composer = newComposerModel()
        _ = composer.replaceText(newText: "Some code")
        _ = composer.codeBlock()
        _ = composer.enter()
        _ = composer.replaceText(newText: "\t")
        _ = composer.replaceText(newText: "more code")
        _ = composer.enter()
        _ = composer.enter()
        XCTAssertEqual(composer.getContentAsHtml(), Constants.resultHtml)
        XCTAssertEqual(composer.toTree(), Constants.resultTree)
    }
}
