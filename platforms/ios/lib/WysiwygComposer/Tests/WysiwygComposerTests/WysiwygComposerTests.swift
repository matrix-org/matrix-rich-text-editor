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

@testable import WysiwygComposer
import XCTest

private enum Constants {
    static let fallbackContent = "Fallback content"
}

final class WysiwygComposerTests: XCTestCase {
    func testComposerEmptyState() {
        ComposerModelWrapper()
            .assertHtml("")
            .execute { XCTAssertEqual($0.getContentAsMarkdown(), "") }
            .assertSelection(start: 0, end: 0)
    }

    func testComposerCrashRecovery() {
        class SomeDelegate: ComposerModelWrapperDelegate {
            func fallbackContent() -> String {
                Constants.fallbackContent
            }
        }

        let delegate = SomeDelegate()
        let model = ComposerModelWrapper()
        model.delegate = delegate

        model
            .action { $0.replaceText(newText: "Some text") }
            // Force a crash
            .action { $0.setContentFromHtml(html: "<//strong>") }
            .assertHtml(Constants.fallbackContent)
    }
}
