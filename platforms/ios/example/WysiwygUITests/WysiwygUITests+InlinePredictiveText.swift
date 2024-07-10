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

import XCTest

extension WysiwygUITests {
    func testInlinePredictiveText() {
        sleep(5)
        // Sometimes autocorrection can break capitalisation, so we need to make sure the first letter is lowercase
        app.keyboards.buttons["shift"].tap()
        app.typeTextCharByCharUsingKeyboard("hello how")
        app.keys["space"].tap()
        sleep(1)
        app.keys["space"].tap()
        assertTextViewContent("hello how are you ")
    }
    
    func testInlinePredictiveTextIgnored() {
        sleep(5)
        // Sometimes autocorrection can break capitalisation, so we need to make sure the first letter is lowercase
        app.keyboards.buttons["shift"].tap()
        app.typeTextCharByCharUsingKeyboard("Hello how")
        button(.sendButton).tap()
        sleep(1)
        assertContentText(plainText: "hello how", htmlText: "hello how")
    }
    
    func testDoubleSpaceIntoDot() {
        sleep(5)
        // Sometimes autocorrection can break capitalisation, so we need to make sure the first letter is lowercase
        app.keyboards.buttons["shift"].tap()
        app.typeTextCharByCharUsingKeyboard("hello")
        app.keys["space"].tap()
        app.keys["space"].tap()
        assertTextViewContent("hello. ")
    }
}

extension XCUIApplication {
    func typeTextCharByCharUsingKeyboard(_ text: String) {
        for char in text {
            if char == " " {
                keys["space"].tap()
                continue
            }
            keys[String(char)].tap()
        }
    }
}
