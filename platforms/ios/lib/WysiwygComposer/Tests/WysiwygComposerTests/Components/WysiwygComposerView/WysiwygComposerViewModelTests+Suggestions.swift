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

import Combine
@testable import WysiwygComposer
import XCTest

extension WysiwygComposerViewModelTests {
    func testAtSuggestionsArePublished() {
        let expectation = expectSuggestionPattern(
            expectedPattern: SuggestionPattern(key: .at, text: "ali", start: 0, end: 4)
        )
        _ = viewModel.replaceText(range: .zero, replacementText: "@ali")
        waitExpectation(expectation: expectation, timeout: 2.0)
        let expectation2 = expectSuggestionPattern(
            expectedPattern: SuggestionPattern(key: .at, text: "alice", start: 0, end: 6)
        )
        _ = viewModel.replaceText(range: .init(location: 4, length: 0), replacementText: "ce")
        waitExpectation(expectation: expectation2, timeout: 2.0)
    }

    func testHashSuggestionsArePublished() {
        let expectation = expectSuggestionPattern(
            expectedPattern: SuggestionPattern(key: .hash, text: "room", start: 0, end: 5)
        )
        _ = viewModel.replaceText(range: .zero, replacementText: "#room")
        waitExpectation(expectation: expectation, timeout: 2.0)
    }

    func testSlashSuggestionArePublished() {
        let expectation = expectSuggestionPattern(
            expectedPattern: SuggestionPattern(key: .slash, text: "inv", start: 0, end: 4)
        )
        _ = viewModel.replaceText(range: .zero, replacementText: "/inv")
        waitExpectation(expectation: expectation, timeout: 2.0)
    }

    func testAtSuggestionCanBeUsed() {
        _ = viewModel.replaceText(range: .zero, replacementText: "@ali")
        viewModel.setMention(url: "https://matrix.to/#/@alice:matrix.org", name: "Alice", mentionType: .user)
        XCTAssertEqual(
            viewModel.content.html,
            """
            <a href="https://matrix.to/#/@alice:matrix.org">Alice</a>\u{00A0}
            """
        )
    }
    
    func testAtRoomSuggestionCanBeUsed() {
        _ = viewModel.replaceText(range: .zero, replacementText: "@ro")
        viewModel.setAtRoomMention()
        XCTAssertEqual(
            viewModel.content.html,
            """
            @room\u{00A0}
            """
        )
    }

    func testAtMentionWithNoSuggestion() {
        _ = viewModel.replaceText(range: .zero, replacementText: "Text")
        viewModel.select(range: .init(location: 0, length: 4))
        viewModel.setMention(url: "https://matrix.to/#/@alice:matrix.org", name: "Alice", mentionType: .user)
        // Text is not removed, and the
        // mention is added after the text
        XCTAssertEqual(
            viewModel.content.html,
            """
            Text<a href="https://matrix.to/#/@alice:matrix.org">Alice</a>\u{00A0}
            """
        )
    }
    
    func testAtRoomMentionWithNoSuggestion() {
        _ = viewModel.replaceText(range: .zero, replacementText: "Text")
        viewModel.select(range: .init(location: 0, length: 4))
        viewModel.setAtRoomMention()
        // Text is not removed, and the
        // mention is added after the text
        XCTAssertEqual(
            viewModel.content.html,
            """
            Text@room\u{00A0}
            """
        )
    }
    
    func testAtMentionWithNoSuggestionAtLeading() {
        _ = viewModel.replaceText(range: .zero, replacementText: "Text")
        viewModel.select(range: .init(location: 0, length: 0))
        viewModel.setMention(url: "https://matrix.to/#/@alice:matrix.org", name: "Alice", mentionType: .user)
        // Text is not removed, and the mention is added before the text
        XCTAssertEqual(
            viewModel.content.html,
            """
            <a href="https://matrix.to/#/@alice:matrix.org">Alice</a>Text
            """
        )
    }
    
    func testAtRoomMentionWithNoSuggestionAtLeading() {
        _ = viewModel.replaceText(range: .zero, replacementText: "Text")
        viewModel.select(range: .init(location: 0, length: 0))
        viewModel.setAtRoomMention()
        // Text is not removed, and the mention is added before the text
        XCTAssertEqual(
            viewModel.content.html,
            """
            @roomText
            """
        )
    }

    func testHashSuggestionCanBeUsed() {
        _ = viewModel.replaceText(range: .zero, replacementText: "#roo")
        viewModel.setMention(url: "https://matrix.to/#/#room1:matrix.org", name: "Room 1", mentionType: .room)
        XCTAssertEqual(
            viewModel.content.html,
            """
            <a href="https://matrix.to/#/#room1:matrix.org">#room1:matrix.org</a>\u{00A0}
            """
        )
    }

    func testHashMentionWithNoSuggestion() {
        _ = viewModel.replaceText(range: .zero, replacementText: "Text")
        viewModel.select(range: .init(location: 0, length: 4))
        viewModel.setMention(url: "https://matrix.to/#/#room1:matrix.org", name: "Room 1", mentionType: .room)
        XCTAssertEqual(
            viewModel.content.html,
            """
            Text<a href="https://matrix.to/#/#room1:matrix.org">#room1:matrix.org</a>\u{00A0}
            """
        )
    }

    func testHashMentionWithNoSuggestionAtLeading() {
        _ = viewModel.replaceText(range: .zero, replacementText: "Text")
        viewModel.select(range: .init(location: 0, length: 0))
        viewModel.setMention(url: "https://matrix.to/#/#room1:matrix.org", name: "Room 1", mentionType: .room)
        XCTAssertEqual(
            viewModel.content.html,
            """
            <a href="https://matrix.to/#/#room1:matrix.org">#room1:matrix.org</a>Text
            """
        )
    }

    func testSlashSuggestionCanBeUsed() {
        _ = viewModel.replaceText(range: .zero, replacementText: "/inv")
        viewModel.setCommand(name: "/invite")
        XCTAssertEqual(
            viewModel.content.html,
            """
            /invite\u{00A0}
            """
        )
    }
}

private extension WysiwygComposerViewModelTests {
    /// Create an expectation for a `SuggestionPattern` to be published by the view model.
    ///
    /// - Parameters:
    ///   - expectedPattern: Expected `SuggestionPattern`.
    ///   - description: Description for expectation.
    /// - Returns: Expectation to be fulfilled. Can be used with `waitExpectation`.
    func expectSuggestionPattern(expectedPattern: SuggestionPattern,
                                 description: String = "Await suggestion pattern") -> WysiwygTestExpectation {
        let expectSuggestionPattern = expectation(description: description)
        let cancellable = viewModel.$suggestionPattern
            // Ignore on subscribe publish.
            .removeDuplicates()
            .dropFirst()
            .sink(receiveValue: { suggestionPattern in
                XCTAssertEqual(
                    suggestionPattern,
                    expectedPattern
                )
                expectSuggestionPattern.fulfill()
            })
        return WysiwygTestExpectation(value: expectSuggestionPattern, cancellable: cancellable)
    }
}
