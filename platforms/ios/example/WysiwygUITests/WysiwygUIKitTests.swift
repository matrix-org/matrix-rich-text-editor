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

class WysiwygUIKitTests: XCTestCase {
    private let app = XCUIApplication(bundleIdentifier: "org.matrix.WysiwygUIKit")

    override func setUpWithError() throws {
        continueAfterFailure = false
        app.launch()
        try WysiwygSharedTests.focusComposerAndClearTutorialIfNeeded(app)
    }

    override func tearDownWithError() throws { }

    func testTypingAndDeleting() throws {
        try WysiwygSharedTests.testTypingAndDeleting(app)
    }

    func testTypingAndBolding() throws {
        let attachment = try WysiwygSharedTests.testTypingAndBolding(app)
        add(attachment)
    }

    func testTypingAndSending() throws {
        try WysiwygSharedTests.typeAndSendMessage(app)
    }
}
