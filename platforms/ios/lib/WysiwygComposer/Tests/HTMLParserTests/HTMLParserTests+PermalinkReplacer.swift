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

@testable import HTMLParser
import XCTest

extension HTMLParserTests {
    func testReplaceLinks() throws {
        let html = "<a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\">Alice</a>:\(String.nbsp)"
        let attributed = try HTMLParser.parse(html: html, mentionReplacer: CustomHTMLMentionReplacer())
        // A text attachment is added.
        XCTAssertTrue(attributed.attribute(.attachment, at: 0, effectiveRange: nil) is NSTextAttachment)
        // The original content is added to the new part of the attributed string.
        let originalContent = attributed.attribute(.mention, at: 0, effectiveRange: nil) as? MentionContent
        XCTAssertEqual(
            originalContent?.rustLength,
            1
        )
        // HTML and attributed range matches
        let htmlRange = NSRange(location: 0, length: 1)
        let attributedRange = NSRange(location: 0, length: 1)
        XCTAssertEqual(
            try attributed.attributedRange(from: htmlRange),
            attributedRange
        )
        XCTAssertEqual(
            try attributed.htmlRange(from: attributedRange),
            htmlRange
        )
        // HTML chars match content.
        XCTAssertEqual(
            attributed.htmlChars,
            "\(String.object):\(String.nbsp)"
        )
    }

    func testMentionsAreNotReplaced() throws {
        let html = "<a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\">Alice</a>:\(String.nbsp)"
        let attributed = try HTMLParser.parse(html: html, mentionReplacer: nil)
        // No text attachment.
        XCTAssertFalse(attributed.attribute(.attachment, at: 0, effectiveRange: nil) is NSTextAttachment)
        // The original content is still added to the new part of the attributed string.
        let originalContent = attributed.attribute(.mention, at: 0, effectiveRange: nil) as? MentionContent
        XCTAssertEqual(
            originalContent?.rustLength,
            1
        )
        // HTML and attributed range matches
        let htmlRange = NSRange(location: 0, length: 1)
        let attributedRange = NSRange(location: 0, length: 5)
        XCTAssertEqual(
            try attributed.attributedRange(from: htmlRange),
            attributedRange
        )
        XCTAssertEqual(
            try attributed.htmlRange(from: attributedRange),
            htmlRange
        )

        // Positions in the middle of the mention should translate to the end of it
        XCTAssertEqual(try attributed.htmlPosition(at: 1), 1)
        XCTAssertEqual(try attributed.htmlPosition(at: 2), 1)
        XCTAssertEqual(try attributed.htmlPosition(at: 3), 1)
        XCTAssertEqual(try attributed.htmlPosition(at: 4), 1)

        // HTML chars match content.
        XCTAssertEqual(
            attributed.htmlChars,
            "\(String.object):\(String.nbsp)"
        )
    }

    func testReplaceMultipleLinks() throws {
        let html = """
        <a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\">Alice</a> \
        <a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\">Alice</a>\(String.nbsp)
        """
        let attributed = try HTMLParser.parse(html: html, mentionReplacer: CustomHTMLMentionReplacer())
        // HTML position matches exactly (Rust model mention length is 1, and so is the length of a pill).
        XCTAssertEqual(try attributed.htmlPosition(at: 0), 0)
        XCTAssertEqual(try attributed.htmlPosition(at: 1), 1)
        XCTAssertEqual(try attributed.htmlPosition(at: 2), 2)
        XCTAssertEqual(try attributed.htmlPosition(at: 3), 3)
        XCTAssertEqual(try attributed.htmlPosition(at: 4), 4)
        // Out of bound attributed position throws
        do {
            _ = try attributed.htmlPosition(at: 5)
            XCTFail("HTML position call should have thrown")
        } catch {
            XCTAssertEqual(error as? AttributedRangeError, AttributedRangeError.outOfBoundsAttributedIndex(index: 5))
        }

        // Attributed position matches exactly (Rust model mention length is 1, and so is the length of a pill).
        XCTAssertEqual(try attributed.attributedPosition(at: 0), 0)
        XCTAssertEqual(try attributed.attributedPosition(at: 1), 1)
        XCTAssertEqual(try attributed.attributedPosition(at: 2), 2)
        XCTAssertEqual(try attributed.attributedPosition(at: 3), 3)
        XCTAssertEqual(try attributed.attributedPosition(at: 4), 4)

        let firstLinkHtmlRange = NSRange(location: 0, length: 1)
        let firstLinkAttributedRange = NSRange(location: 0, length: 1)
        XCTAssertEqual(
            try attributed.attributedRange(from: firstLinkHtmlRange),
            firstLinkAttributedRange
        )
        XCTAssertEqual(
            try attributed.htmlRange(from: firstLinkAttributedRange),
            firstLinkHtmlRange
        )

        let secondLinkHtmlRange = NSRange(location: 2, length: 1)
        let secondLinkAttributedRange = NSRange(location: 2, length: 1)
        XCTAssertEqual(
            try attributed.attributedRange(from: secondLinkHtmlRange),
            secondLinkAttributedRange
        )
        XCTAssertEqual(
            try attributed.htmlRange(from: secondLinkAttributedRange),
            secondLinkHtmlRange
        )
        // HTML chars match content.
        XCTAssertEqual(
            attributed.htmlChars,
            "\(String.object) \(String.object)\(String.nbsp)"
        )
    }

    func testMultipleMentionsAreNotReplaced() throws {
        let html = """
        <a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\">Alice</a> \
        <a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\">Alice</a>\(String.nbsp)
        """
        let attributed = try HTMLParser.parse(html: html, mentionReplacer: nil)
        // HTML position matches.
        XCTAssertEqual(try attributed.htmlPosition(at: 0), 0)
        XCTAssertEqual(try attributed.htmlPosition(at: 5), 1)
        XCTAssertEqual(try attributed.htmlPosition(at: 6), 2)
        XCTAssertEqual(try attributed.htmlPosition(at: 11), 3)
        XCTAssertEqual(try attributed.htmlPosition(at: 12), 4)
        // Out of bound attributed position throws
        do {
            _ = try attributed.htmlPosition(at: 13)
            XCTFail("HTML position call should have thrown")
        } catch {
            XCTAssertEqual(error as? AttributedRangeError, AttributedRangeError.outOfBoundsAttributedIndex(index: 13))
        }

        // Attributed position matches.
        XCTAssertEqual(try attributed.attributedPosition(at: 0), 0)
        XCTAssertEqual(try attributed.attributedPosition(at: 1), 5)
        XCTAssertEqual(try attributed.attributedPosition(at: 2), 6)
        XCTAssertEqual(try attributed.attributedPosition(at: 3), 11)
        XCTAssertEqual(try attributed.attributedPosition(at: 4), 12)

        let firstLinkHtmlRange = NSRange(location: 0, length: 1)
        let firstLinkAttributedRange = NSRange(location: 0, length: 5)
        XCTAssertEqual(
            try attributed.attributedRange(from: firstLinkHtmlRange),
            firstLinkAttributedRange
        )
        XCTAssertEqual(
            try attributed.htmlRange(from: firstLinkAttributedRange),
            firstLinkHtmlRange
        )

        let secondLinkHtmlRange = NSRange(location: 2, length: 1)
        let secondLinkAttributedRange = NSRange(location: 6, length: 5)
        XCTAssertEqual(
            try attributed.attributedRange(from: secondLinkHtmlRange),
            secondLinkAttributedRange
        )
        XCTAssertEqual(
            try attributed.htmlRange(from: secondLinkAttributedRange),
            secondLinkHtmlRange
        )
        // HTML chars match content.
        XCTAssertEqual(
            attributed.htmlChars,
            "\(String.object) \(String.object)\(String.nbsp)"
        )
    }
}

private class CustomHTMLMentionReplacer: HTMLMentionReplacer {
    func replacementForMention(_ url: String, text: String) -> NSAttributedString? {
        if url.starts(with: "https://matrix.to/#/"),
           let image = UIImage(systemName: "link") {
            // Set a text attachment with an arbitrary image.
            return NSAttributedString(attachment: NSTextAttachment(image: image))
        } else {
            return nil
        }
    }
}
