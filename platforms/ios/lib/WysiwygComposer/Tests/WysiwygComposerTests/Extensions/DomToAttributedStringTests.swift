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

import CoreText
@testable import WysiwygComposer
import XCTest

final class DomToAttributedStringTests: XCTestCase {
    func testSimpleTextCase() throws {
        let dom: DomNode = .container(
            path: [],
            kind: .generic,
            children: [
                .text(path: [], text: "foo"),
                .container(path: [], kind: .formatting(.bold), children: [.text(path: [], text: " bold")]),
                .text(path: [], text: " bar"),
            ]
        )
        XCTAssertEqual(NSAttributedString(dom.toAttributedText).string, "foo bold bar")
    }
    
    func testFormattedTextAttributes() throws {
        let dom: DomNode = .container(
            path: [],
            kind: .generic,
            children: [
                .text(path: [], text: "Some"),
                .container(path: [], kind: .formatting(.bold), children: [
                    .text(path: [], text: " bold and"),
                    .container(path: [], kind: .formatting(.italic), children: [
                        .text(path: [], text: " italic"),
                    ]),
                ]),
                .text(path: [], text: " text"),
            ]
        )
        let attributed = NSAttributedString(dom.toAttributedText)
        
        // Font at index 6 is bold
        let fontTraits1 = attributed.fontSymbolicTraits(at: 6)
        XCTAssert(fontTraits1.contains(.traitBold))
        XCTAssert(!fontTraits1.contains(.traitItalic))
        // Font at index 15 is bold and italic
        let fontTraits2 = attributed.fontSymbolicTraits(at: 15)
        print(fontTraits2)
        
        let a: CTFontSymbolicTraits = [.traitBold, .traitItalic]
        print(a)
        XCTAssert(fontTraits2.isSuperset(of: [.traitBold, .traitItalic]))
        // Font at index 2 is neither italic, nor bold
        let fontTraits3 = attributed.fontSymbolicTraits(at: 2)
        XCTAssert(fontTraits3.isDisjoint(with: [.traitBold, .traitItalic]))
    }
}
