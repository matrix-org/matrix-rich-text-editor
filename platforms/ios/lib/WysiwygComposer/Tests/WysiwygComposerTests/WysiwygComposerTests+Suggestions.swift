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
    func testSuggestionForAtPattern() {
        let model = ComposerModelWrapper()
        let update = model.replaceText(newText: "@alic")

        guard case .suggestion(suggestionPattern: let suggestionPattern) = update.menuAction(),
              let attributes = suggestionPattern.key.mentionType?.attributes
        else {
            XCTFail("No user suggestion found")
            return
        }

        model
            .action {
                $0.insertMentionAtSuggestion(
                    url: "https://matrix.to/#/@alice:matrix.org",
                    text: "Alice",
                    suggestion: suggestionPattern,
                    attributes: attributes
                )
            }
            .assertHtml(
                """
                <a data-mention-type="user" href="https://matrix.to/#/@alice:matrix.org" contenteditable="false">Alice</a>\(String.nbsp)
                """
            )
            .assertSelection(start: 2, end: 2)
    }

    func testNonLeadingSuggestionForAtPattern() {
        let model = ComposerModelWrapper()
        let update = model.replaceText(newText: "Hello @alic")

        guard case .suggestion(suggestionPattern: let suggestionPattern) = update.menuAction(),
              let attributes = suggestionPattern.key.mentionType?.attributes
        else {
            XCTFail("No user suggestion found")
            return
        }

        model
            .action {
                $0.insertMentionAtSuggestion(
                    url: "https://matrix.to/#/@alice:matrix.org",
                    text: "Alice",
                    suggestion: suggestionPattern,
                    attributes: attributes
                )
            }
            .assertHtml(
                """
                Hello <a data-mention-type="user" \
                href="https://matrix.to/#/@alice:matrix.org" \
                contenteditable="false">Alice</a>\(String.nbsp)
                """
            )
            .assertSelection(start: 8, end: 8)
    }

    func testSuggestionForHashPattern() {
        let model = ComposerModelWrapper()
        let update = model.replaceText(newText: "#roo")

        guard case .suggestion(suggestionPattern: let suggestionPattern) = update.menuAction(),
              let attributes = suggestionPattern.key.mentionType?.attributes
        else {
            XCTFail("No room suggestion found")
            return
        }

        model
            .action {
                $0.insertMentionAtSuggestion(
                    url: "https://matrix.to/#/#room1:matrix.org",
                    text: "Room 1",
                    suggestion: suggestionPattern,
                    attributes: attributes
                )
            }
            .assertHtml(
                """
                <a data-mention-type="room" href="https://matrix.to/#/#room1:matrix.org" contenteditable="false">Room 1</a>\(String.nbsp)
                """
            )
    }

    func testSuggestionForSlashPattern() {
        let model = ComposerModelWrapper()
        let update = model.replaceText(newText: "/")

        guard case .suggestion(suggestionPattern: let suggestionPattern) = update.menuAction() else {
            XCTFail("No suggestion found")
            return
        }

        model
            .action {
                $0.replaceTextSuggestion(newText: "/invite", suggestion: suggestionPattern)
            }
            .assertHtml("/invite\(String.nbsp)")
    }
}
