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
    func testCodeBlock() throws {
        // Type something into composer.
        textView.typeTextCharByChar("Some text")
        button(.codeBlockButton).tap()
        // FIXME: iOS automatically adds an extra line break even if not in the model
        assertTextViewContent("​Some text\n")

        button(.showTreeButton).tap()
        assertTreeEquals(
            """
            └>pre
              ├>~
              └>"Some text"
            """
        )
    }
}
