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
        newComposerModel()
            .action { $0.replaceText(newText: TestConstants.testStringWithEmojis) }
            // Text is preserved, including emojis.
            .assertHtml(TestConstants.testStringWithEmojis)
            // Selection is set at the end of the text.
            .assertSelection(start: 14, end: 14)
    }

    func testBackspacingEmoji() {
        newComposerModel()
            .action { $0.replaceText(newText: TestConstants.testStringWithEmojis) }
            .action { $0.select(startUtf16Codeunit: 7, endUtf16Codeunit: 14) }
            .action { $0.backspace() }
            // Text should remove exactly the last emoji.
            .assertHtml(TestConstants.testStringAfterBackspace)
            .assertSelection(start: 7, end: 7)
    }
}
