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

extension WysiwygComposerTests {
    func testLists() {
        ComposerModelWrapper()
            .action { $0.apply(.orderedList) }
            .action { $0.replaceText(newText: "Item 1") }
            .action { $0.enter() }
            .action { $0.replaceText(newText: "Item 2") }
            // Add a third list item
            .action { $0.enter() }
            .assertHtml("<ol><li>Item 1</li><li>Item 2</li><li></li></ol>")
            .assertSelection(start: 14, end: 14)
            // Remove it
            .action { $0.enter() }
            .assertHtml("<ol><li>Item 1</li><li>Item 2</li></ol><p>\(Character.nbsp)</p>")
            .assertSelection(start: 14, end: 14)
            // Insert some text afterwards
            .action { $0.replaceText(newText: "Some text") }
            .assertHtml("<ol><li>Item 1</li><li>Item 2</li></ol><p>Some text</p>")
            .assertSelection(start: 23, end: 23)
    }
}
