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
    func testLists() {
        let composer = newComposerModel()
        _ = composer.orderedList()
        _ = composer.replaceText(newText: "Item 1")
        _ = composer.enter()
        _ = composer.replaceText(newText: "Item 2")
        // Add a thirs list item
        _ = composer.enter()
        XCTAssertEqual(composer.getContentAsHtml(),
                       "<ol><li>"
                           + TestConstants.zwsp
                           + "Item 1</li><li>"
                           + TestConstants.zwsp
                           + "Item 2</li><li>"
                           + TestConstants.zwsp
                           + "</li></ol>")
        XCTAssertEqual(composer.getCurrentDomState().start, composer.getCurrentDomState().end)
        XCTAssertEqual(composer.getCurrentDomState().start, 15)
        // Remove it
        _ = composer.enter()
        XCTAssertEqual(composer.getContentAsHtml(),
                       "<ol><li>"
                           + TestConstants.zwsp
                           + "Item 1</li><li>"
                           + TestConstants.zwsp
                           + "Item 2</li></ol>"
                           + TestConstants.zwsp)
        XCTAssertEqual(composer.getCurrentDomState().start, composer.getCurrentDomState().end)
        XCTAssertEqual(composer.getCurrentDomState().start, 15)
        // Insert some text afterwards
        _ = composer.replaceText(newText: "Some text")
        XCTAssertEqual(composer.getContentAsHtml(),
                       "<ol><li>"
                           + TestConstants.zwsp
                           + "Item 1</li><li>"
                           + TestConstants.zwsp
                           + "Item 2</li></ol>"
                           + TestConstants.zwsp
                           + "Some text")
        XCTAssertEqual(composer.getCurrentDomState().start, composer.getCurrentDomState().end)
        XCTAssertEqual(composer.getCurrentDomState().start, 24)
    }
}
