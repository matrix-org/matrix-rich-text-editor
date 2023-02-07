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
    static let resultHtml = "<pre><code>Some code\n\tmore code</code></pre><p>\(Character.nbsp)</p>"
    static let resultTree =
        """

        ├>codeblock
        │ ├>p
        │ │ └>"Some code"
        │ └>p
        │   └>"\tmore code"
        └>p

        """
}

extension WysiwygComposerTests {
    func testCodeBlocksFromEmptyComposer() {
        newComposerModel()
            .action { $0.codeBlock() }
            .action { $0.replaceText(newText: "Some code") }
            .action { $0.enter() }
            .action { $0.replaceText(newText: "\t") }
            .action { $0.replaceText(newText: "more code") }
            .action { $0.enter() }
            .action { $0.enter() }
            .assertHtml(Constants.resultHtml)
            .assertTree(Constants.resultTree)
    }

    func testCodeBlocksWithMultilineInput() {
        newComposerModel()
            .action { $0.codeBlock() }
            .action { $0.replaceText(newText: "Some code\n\tmore code") }
            .action { $0.enter() }
            .action { $0.enter() }
            .assertHtml(Constants.resultHtml)
            .assertTree(Constants.resultTree)
    }

    func testCodeBlocksFromContent() {
        newComposerModel()
            .action { $0.replaceText(newText: "Some code") }
            .action { $0.codeBlock() }
            .action { $0.enter() }
            .action { $0.replaceText(newText: "\t") }
            .action { $0.replaceText(newText: "more code") }
            .action { $0.enter() }
            .action { $0.enter() }
            .assertHtml(Constants.resultHtml)
            .assertTree(Constants.resultTree)
    }
}
