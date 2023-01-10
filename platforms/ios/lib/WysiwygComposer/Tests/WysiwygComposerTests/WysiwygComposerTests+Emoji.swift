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
    func testSetBaseStringWithEmoji() {
        let composer = newComposerModel()
        let update = composer.replaceText(newText: TestConstants.testStringWithEmojis)
        switch update.textUpdate() {
        case .keep, .select:
            XCTFail("Expected replace all HTML update")
        case let .replaceAll(replacementHtml: codeUnits,
                             startUtf16Codeunit: start,
                             endUtf16Codeunit: end):
            // Text is preserved, including emojis.
            XCTAssertEqual(String(utf16CodeUnits: codeUnits, count: codeUnits.count),
                           TestConstants.testStringWithEmojis)
            // Selection is set at the end of the text.
            XCTAssertEqual(start, end)
            XCTAssertEqual(end, 14)
        }
    }

    func testBackspacingEmoji() {
        let composer = newComposerModel()
        _ = composer.replaceText(newText: TestConstants.testStringWithEmojis)

        _ = composer.select(startUtf16Codeunit: 7, endUtf16Codeunit: 14)

        let update = composer.backspace()
        switch update.textUpdate() {
        case .keep, .select:
            XCTFail("Expected replace all HTML update")
        case let .replaceAll(replacementHtml: codeUnits,
                             startUtf16Codeunit: start,
                             endUtf16Codeunit: end):
            // Text should remove exactly the last emoji.
            XCTAssertEqual(String(utf16CodeUnits: codeUnits, count: codeUnits.count),
                           TestConstants.testStringAfterBackspace)
            XCTAssertEqual(start, end)
            XCTAssertEqual(start, 7)
        }
    }
}
