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

// These tests work on the assunmption that we always have the software keyboard enabled which is handled through a build phase run script.
// The following tests may also require specific keyboard languages that will be automatically added if needed.
extension WysiwygUITests {
    func testInlinePredictiveText() {
        sleep(1)
        setupKeyboard(.englishQWERTY)
        
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
        sleep(1)
        setupKeyboard(.englishQWERTY)

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
        sleep(1)
        setupKeyboard(.englishQWERTY)

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
        sleep(1)
        setupKeyboard(.englishQWERTY)

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
        sleep(1)
        setupKeyboard(.englishQWERTY)

        // Sometimes autocorrection can break capitalisation, so we need to make sure the first letter is lowercase
        app.keyboards.buttons["shift"].tap()
        app.typeTextCharByCharUsingKeyboard("hello how a")
        // We assert both the tree and textview content because the text view is containing the predictive text at that moment
        // Which in the ui test is seen as part of the static text
        assertTextViewContent("hello how are you")
        app.keys["space"].tap()
        app.keys["more"].tap()
        app.keys["."].tap()
        
        // This optimisation to predictive inline text was introduced in 17.5
        let correctText: String
        if #available(iOS 17.5, *) {
            correctText = "hello how are you."
        } else {
            correctText = "hello how are you ."
        }
        assertTextViewContent(correctText)
        // In the failure case a second dot is added in the tree.
        assertTreeEquals(
            """
            └>"\(correctText)"
            """
        )
    }
    
    func testJapaneseKanaDeletion() {
        sleep(1)
        setupKeyboard(.japaneseKana)

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
    
    private func setupKeyboard(_ keyboard: TestKeyboard) {
        var changeKeyboardButton: XCUIElement!
        let nextKeyboard = app.buttons["Next keyboard"]
        let emoji = app.buttons["Emoji"]
        if nextKeyboard.exists {
            changeKeyboardButton = nextKeyboard
        } else if emoji.exists {
            changeKeyboardButton = emoji
        }
        
        if changeKeyboardButton == nil {
            addKeyboardToSettings(keyboard: keyboard)
            changeKeyboardButton = app.buttons["Next keyboard"]
        }
        
        changeKeyboardButton.press(forDuration: 1)
        var keyboardSelection = app.tables.staticTexts[keyboard.label]
        if !keyboardSelection.exists {
            addKeyboardToSettings(keyboard: keyboard)
            // No need to tap since it gets selected automatically
        } else {
            keyboardSelection.tap()
        }
    }
    
    private func addKeyboardToSettings(keyboard: TestKeyboard) {
        let settingsApp = XCUIApplication(bundleIdentifier: "com.apple.Preferences")
        settingsApp.launch()
        
        settingsApp.tables.cells.staticTexts["General"].tap()
        settingsApp.tables.cells.staticTexts["Keyboard"].tap()
        settingsApp.tables.cells.staticTexts["Keyboards"].tap()
        if settingsApp.tables.cells.staticTexts[keyboard.keyboardIdentifier].exists {
            return
        }
        settingsApp.tables.cells.staticTexts["AddNewKeyboard"].tap()
        settingsApp.tables.cells.staticTexts[keyboard.localeIdentifier].tap()
        if keyboard.hasSubSelection {
            settingsApp.tables.cells.staticTexts[keyboard.keyboardIdentifier].tap()
        }
        settingsApp.buttons["Done"].tap()
        
        settingsApp.terminate()
        app.activate()
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
}

private enum TestKeyboard {
    case englishQWERTY
    case japaneseKana
    
    var keyboardIdentifier: String {
        switch self {
        case .englishQWERTY:
            return "en_US@sw=QWERTY;hw=Automatic"
        case .japaneseKana:
            return "ja_JP-Kana@sw=Kana;hw=Automatic"
        }
    }
    
    var localeIdentifier: String {
        switch self {
        case .englishQWERTY:
            return "en_US"
        case .japaneseKana:
            return "ja_JP"
        }
    }
    
    var label: String {
        switch self {
        case .englishQWERTY:
            return "English (US)"
        case .japaneseKana:
            return "日本語かな"
        }
    }
    
    var hasSubSelection: Bool {
        switch self {
        case .englishQWERTY:
            return false
        case .japaneseKana:
            return true
        }
    }
}
