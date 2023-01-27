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

import HTMLParser
@testable import WysiwygComposer
import XCTest

extension WysiwygComposerTests {
    func testFormatBold() throws {
        newComposerModel()
            .action { $0.replaceText(newText: "This is bold text") }
            .action { $0.select(startUtf16Codeunit: 8, endUtf16Codeunit: 12) }
            .action { $0.bold() }
            .assertHtml("This is <strong>bold</strong> text")
            // Selection is kept after format.
            .assertSelection(start: 8, end: 12)
            .execute {
                // Constructed attributed string sets bold on the selected range.
                guard let attributed = try? HTMLParser.parse(html: $0.getContentAsHtml()) else {
                    XCTFail("Parsing unexpectedly failed")
                    return
                }
                attributed.enumerateTypedAttribute(.font, in: .init(location: 8, length: 4)) { (font: UIFont, range, _) in
                    XCTAssertEqual(range, .init(location: 8, length: 4))
                    XCTAssertTrue(font.fontDescriptor.symbolicTraits.contains(.traitBold))
                }
            }
            .assertTree(
                """

                ├>\"This is \"
                ├>strong
                │ └>\"bold\"
                └>\" text\"

                """
            )
    }
}
