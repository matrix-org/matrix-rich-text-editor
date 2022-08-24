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

import UIKit
import XCTest
@testable import WysiwygComposer

final class UITextViewTests: XCTestCase {
    func testTextViewUTF16Encoding() throws {
        let textView = UITextView()
        textView.attributedText = try NSAttributedString(html: TestConstants.testStringWithEmojis)
        // Selection is at the end of the text, with a UTF-16 length of 14.
        XCTAssertEqual(textView.selectedRange, NSRange(location: 14, length: 0))
        // Text count what is perceived as character.
        XCTAssertEqual(textView.text.count, 6)
        XCTAssertEqual(textView.text.utf16.count, 14)
        // AttributedString counts UTF-16 directly
        XCTAssertEqual(textView.attributedText.length, 14)
        // Test deleting the latest emoji.
        textView.deleteBackward()
        XCTAssertEqual(textView.attributedText.length, 7)
        XCTAssertEqual(textView.text, TestConstants.testStringAfterBackspace)
    }
}
