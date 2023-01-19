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
    /// Type a text and make it bold in the composer.
    /// A screenshot is saved since string attributes can't be read from this context.
    func testTypingAndBolding() throws {
        // Type something into composer.
        textView.typeTextCharByChar("Some bold text")

        textView.doubleTap()
        // We can't detect data being properly reported back to the model but
        // 1s is more than enough for the Rust side to get notified for the selection.
        sleep(1)

        button(.boldButton).tap()
        // Bolding doesn't change text and we can't test text attributes from this context.
        assertTextViewContent("Some bold text")

        // FIXME: an unwanted space is added into the model
        assertTreeEquals(
            """
            ├>"Some bold "
            ├>strong
            │ └>"text"
            └>"  "
            """
        )
    }
}
