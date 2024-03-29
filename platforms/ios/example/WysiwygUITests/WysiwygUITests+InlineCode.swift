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

import XCTest

extension WysiwygUITests {
    func testTypingInlineCodeDisablesOtherFormatters() {
        button(.inlineCodeButton).tap()
        textView.typeTextCharByChar("code")
        let reactiveButtonsIdentifiers: [WysiwygSharedAccessibilityIdentifier] = [
            .boldButton,
            .italicButton,
            .strikeThroughButton,
            .linkButton,
            // FIXME: this should contain other incompatible buttons when Rust is ready
        ]
        for identifier in reactiveButtonsIdentifiers {
            XCTAssertFalse(button(identifier).isEnabled)
        }
        // Inline code is enabled
        XCTAssertTrue(button(.inlineCodeButton).isEnabled)
    }
}
