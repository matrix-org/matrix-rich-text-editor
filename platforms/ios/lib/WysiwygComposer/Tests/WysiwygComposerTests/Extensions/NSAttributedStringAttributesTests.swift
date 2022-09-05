// 
// Copyright 2022 The Matrix.org Foundation C.I.C
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

import XCTest
@testable import WysiwygComposer

final class NSAttributedStringTests: XCTestCase {
    func testEnumerateTypedAttribute() {
        let attributed = NSMutableAttributedString(string: "Test",
                                                   attributes: [.font: UIFont.boldSystemFont(ofSize: 15)])
        attributed.enumerateTypedAttribute(.font) { (font: UIFont, range: NSRange, _) in
            XCTAssertTrue(font.fontDescriptor.symbolicTraits.contains(.traitBold))
            XCTAssertTrue(range == .init(location: 0, length: attributed.length))
        }
        attributed.addAttribute(.font, value: "bad type", range: .init(location: 2, length: 2))
        attributed.enumerateTypedAttribute(.font) { (font: UIFont, range: NSRange, _) in
            XCTAssertTrue(font.fontDescriptor.symbolicTraits.contains(.traitBold))
            XCTAssertTrue(range == .init(location: 0, length: 2))
        }
    }
}
