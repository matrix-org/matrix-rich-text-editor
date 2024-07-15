//
// Copyright 2024 The Matrix.org Foundation C.I.C
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

final class DomToAttributedStringTests: XCTestCase {
    func testSimpleTextCase() throws {
        let dom: DomNode = .container(
            id: 0,
            kind: .generic,
            children: [
                .text(id: 1, text: "foo"),
                .container(id: 2, kind: .formatting(.bold), children: [.text(id: 3, text: " bold")]),
                .text(id: 4, text: " bar"),
            ]
        )
        XCTAssertEqual(dom.toAttributedText, NSAttributedString(string: "foo bold bar"))
    }
}
