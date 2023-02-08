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
    static let resultHtml = "<blockquote><p>Some quote</p><p>More text</p></blockquote><p>\(Character.nbsp)</p>"
    static let resultTree =
        """

        ├>blockquote
        │ ├>p
        │ │ └>"Some quote"
        │ └>p
        │   └>"More text"
        └>p

        """
}

extension WysiwygComposerTests {
    func testQuotesFromEmptyComposer() {
        ComposerModelWrapper()
            .action { $0.apply(.quote) }
            .action { $0.replaceText(newText: "Some quote") }
            .action { $0.enter() }
            .action { $0.replaceText(newText: "More text") }
            .action { $0.enter() }
            .action { $0.enter() }
            .assertHtml(Constants.resultHtml)
            .assertTree(Constants.resultTree)
    }

    func testQuotesWithMultilineInput() {
        ComposerModelWrapper()
            .action { $0.apply(.quote) }
            .action { $0.replaceText(newText: "Some quote\nMore text") }
            .action { $0.enter() }
            .action { $0.enter() }
            .assertHtml(Constants.resultHtml)
            .assertTree(Constants.resultTree)
    }

    func testQuotesFromContent() {
        ComposerModelWrapper()
            .action { $0.replaceText(newText: "Some quote") }
            .action { $0.apply(.quote) }
            .action { $0.enter() }
            .action { $0.replaceText(newText: "More text") }
            .action { $0.enter() }
            .action { $0.enter() }
            .assertHtml(Constants.resultHtml)
            .assertTree(Constants.resultTree)
    }
}
