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

// These tests work on the assunmption that we always have the software keyboard enabled
// which is handled through the run script
// and that the simulator is configured with the following 2 keyboards: English (US) and Japanese Kana
// Such configuration is done through a script called ios-keyboard-setup.sh but can also be done manually
extension WysiwygUITests {
    func testInlinePredictiveText() {
        sleep(3)
        app.setupKeyboardLanguage(named: "English (US)")
        
        // Sometimes autocorrection can break capitalisation, so we need to make sure the first letter is lowercase
        app.keyboards.buttons["shift"].tap()
        app.typeTextCharByCharUsingKeyboard("hello how a")
        // We assert both the tree and textview content because the text view is containing the predictive text at that moment
        // Which in the ui test is seen as part of the static text
        assertTextViewContent("hello how are you")
        assertTreeEquals(
            """
            └>"hello how a"
            """
        )
        app.keys["space"].tap()
        assertTextViewContent("hello how are you ")
        assertTreeEquals(
            """
            └>"hello how are you "
            """
        )
    }
    
    func testInlinePredictiveTextIsIgnoredWhenSending() {
        sleep(3)
        app.setupKeyboardLanguage(named: "English (US)")

        // Sometimes autocorrection can break capitalisation, so we need to make sure the first letter is lowercase
        app.keyboards.buttons["shift"].tap()
        app.typeTextCharByCharUsingKeyboard("hello how")
        // We assert both the tree and textview content because the text view is containing the predictive text at that moment
        // Which in the ui test is seen as part of the static text
        assertTextViewContent("hello how are you")
        assertTreeEquals(
            """
            └>"hello how"
            """
        )
        button(.sendButton).tap()
        sleep(1)
        assertContentText(plainText: "hello how", htmlText: "hello how")
    }
    
    func testInlinePredictiveTextIsIgnoredWhenDeleting() {
        sleep(3)
        app.setupKeyboardLanguage(named: "English (US)")

        // Sometimes autocorrection can break capitalisation, so we need to make sure the first letter is lowercase
        app.keyboards.buttons["shift"].tap()
        app.typeTextCharByCharUsingKeyboard("hello how")
        app.keys["delete"].tap()
        // We assert both the tree and textview content because the text view is containing the predictive text at that moment
        // Which in the ui test is seen as part of the static text
        assertTextViewContent("hello how are you")
        assertTreeEquals(
            """
            └>"hello ho"
            """
        )
        button(.sendButton).tap()
        sleep(1)
        assertContentText(plainText: "hello ho", htmlText: "hello ho")
    }
    
    func testDoubleSpaceIntoDot() {
        sleep(3)
        app.setupKeyboardLanguage(named: "English (US)")

        // Sometimes autocorrection can break capitalisation, so we need to make sure the first letter is lowercase
        app.keyboards.buttons["shift"].tap()
        app.typeTextCharByCharUsingKeyboard("hello")
        app.keys["space"].tap()
        app.keys["space"].tap()
        assertTextViewContent("hello. ")
        assertTreeEquals(
            """
            └>"hello. "
            """
        )
    }
    
    func testDotAfterInlinePredictiveText() {
        sleep(3)
        app.setupKeyboardLanguage(named: "English (US)")

        // Sometimes autocorrection can break capitalisation, so we need to make sure the first letter is lowercase
        app.keyboards.buttons["shift"].tap()
        app.typeTextCharByCharUsingKeyboard("hello how a")
        // We assert both the tree and textview content because the text view is containing the predictive text at that moment
        // Which in the ui test is seen as part of the static text
        assertTextViewContent("hello how are you")
        app.keys["space"].tap()
        app.keys["more"].tap()
        app.keys["."].tap()
        assertTextViewContent("hello how are you.")
        // In the failure case a second dot is added in the tree.
        assertTreeEquals(
            """
            └>"hello how are you."
            """
        )
    }
    
    func testJapaneseKanaDeletion() {
        sleep(3)
        app.setupKeyboardLanguage(named: "日本語かな")

        app.typeTextCharByCharUsingKeyboard("は")
        assertTextViewContent("は")
        assertTreeEquals(
            """
            └>"は"
            """
        )
        app.keys["delete"].tap()
        assertTextViewContent("")
        XCTAssertEqual(staticText(.treeText).label, "\n")
    }
}

private extension XCUIApplication {
    func typeTextCharByCharUsingKeyboard(_ text: String) {
        for char in text {
            if char == " " {
                keys["space"].tap()
                continue
            }
            keys[String(char)].tap()
        }
    }
    
    func setupKeyboardLanguage(named language: String) {
        let nextKeyboard = buttons["Next keyboard"]
        while let value = nextKeyboard.value as? String,
              value != language {
            nextKeyboard.tap()
        }
        nextKeyboard.tap()
    }
}