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

@testable import WysiwygComposer
import XCTest

final class NSAttributedStringRangeTests: XCTestCase {
    let zwsp = "\u{200B}"
    
    func testAttributedNumberedLists() throws {
        let html = "<ol><li>\(zwsp)Item 1</li><li>\(zwsp)Item 2</li></ol>Some Text"
        let attributed = try NSAttributedString(html: html)
        // A textual representation of the numbered list is displayed
        XCTAssertEqual(attributed.string,
                       "\t1.\tItem 1\n\t2.\tItem 2\nSome Text")
        // Ranges that are not part of the raw HTML text (excluding tags) are detected
        XCTAssertEqual(attributed.listPrefixesRanges(),
                       [NSRange(location: 0, length: 4),
                        NSRange(location: 11, length: 4)])
        // Converting back and forth from HTML to attributed postions
        XCTAssertEqual(try attributed.htmlPosition(at: 4), 1)
        XCTAssertEqual(try attributed.attributedPosition(at: 1), 4)
        XCTAssertEqual(try attributed.htmlPosition(at: 10), 7)
        XCTAssertEqual(try attributed.attributedPosition(at: 7), 10)
        XCTAssertEqual(try attributed.htmlPosition(at: 15), 8)
        XCTAssertEqual(try attributed.attributedPosition(at: 8), 15)

        // Attributed index inside a prefix should return a valid index in the HTML raw text
        XCTAssertEqual(attributed.character(at: 11), "\t")
        XCTAssertEqual(try attributed.htmlPosition(at: 11), 8)
        XCTAssertEqual(attributed.character(at: 12), "2")
        XCTAssertEqual(try attributed.htmlPosition(at: 12), 8)

        // Converting back and forth from HTML to attributed ranges
        // Both expected range for "Item 1"
        let htmlRange = NSRange(location: 1, length: 6)
        let attributedRange = NSRange(location: 4, length: 6)
        XCTAssertEqual(try attributed.attributedRange(from: htmlRange),
                       attributedRange)
        XCTAssertEqual(try attributed.htmlRange(from: attributedRange),
                       htmlRange)
        XCTAssertEqual(attributed.attributedSubstring(from: attributedRange).string,
                       "Item 1")

        // Cross list items range
        let crossHtmlRange = NSRange(location: 1, length: 8)
        let crossAttributedRange = NSRange(location: 4, length: 12)
        XCTAssertEqual(try attributed.attributedRange(from: crossHtmlRange),
                       crossAttributedRange)
        XCTAssertEqual(try attributed.htmlRange(from: crossAttributedRange),
                       crossHtmlRange)
        XCTAssertEqual(attributed.attributedSubstring(from: crossAttributedRange).string,
                       "Item 1\n\t2.\tI")
    }

    func testAttributedBulletedLists() throws {
        let html = "<ul><li>\(zwsp)Item 1</li><li>\(zwsp)Item 2</li></ul>Some Text"
        let attributed = try NSAttributedString(html: html)
        XCTAssertEqual(attributed.string,
                       "\t•\tItem 1\n\t•\tItem 2\nSome Text")
        XCTAssertEqual(attributed.listPrefixesRanges(),
                       [NSRange(location: 0, length: 3),
                        NSRange(location: 10, length: 3)])
        XCTAssertEqual(try attributed.attributedPosition(at: 1), 3)
        XCTAssertEqual(try attributed.attributedPosition(at: 8), 13)
        XCTAssertEqual(try attributed.htmlPosition(at: 13), 8)
        XCTAssertEqual(try attributed.htmlPosition(at: 3), 1)
    }

    func testMultipleAttributedLists() throws {
        let html = "<ol><li>\(zwsp)Item 1</li><li>\(zwsp)Item 2</li></ol><ul><li>\(zwsp)Item 1</li><li>\(zwsp)Item 2</li></ul>"
        let attributed = try NSAttributedString(html: html)
        XCTAssertEqual(attributed.string,
                       "\t1.\tItem 1\n\t2.\tItem 2\n\t•\tItem 1\n\t•\tItem 2\n")
        XCTAssertEqual(attributed.listPrefixesRanges(),
                       [NSRange(location: 0, length: 4),
                        NSRange(location: 11, length: 4),
                        NSRange(location: 22, length: 3),
                        NSRange(location: 32, length: 3)])
        XCTAssertEqual(try attributed.attributedPosition(at: 14), 21)
        XCTAssertEqual(try attributed.htmlPosition(at: 21), 14)
        XCTAssertEqual(try attributed.attributedRange(from: .init(location: 0, length: 12)),
                       NSRange(location: 0, length: 19))
        XCTAssertEqual(try attributed.htmlRange(from: .init(location: 4, length: 17)),
                       NSRange(location: 1, length: 13))
    }

    func testMultipleDigitsNumberedLists() throws {
        var html = "<ol>"
        for _ in 1...100 {
            html.append(contentsOf: "<li>abcd</li>")
        }
        html.append(contentsOf: "</ol>")
        let attributed = try NSAttributedString(html: html)
        XCTAssertEqual(attributed.listPrefixesRanges().count,
                       100)
    }

    func testOutOfBoundsIndexes() throws {
        let html = "<ol><li>Item 1</li><li>Item 2</li></ol>Some Text"
        let attributed = try NSAttributedString(html: html)
        // Out of bounds indexes return errors
        do {
            _ = try attributed.attributedPosition(at: 40)
        } catch {
            XCTAssertEqual(error as? AttributedRangeError,
                           AttributedRangeError.outOfBoundsHtmlIndex(index: 40))
            XCTAssertEqual(error.localizedDescription,
                           "Provided HTML index is out of expected bounds (40)")
        }
        do {
            _ = try attributed.htmlPosition(at: 50)
        } catch {
            XCTAssertEqual(error as? AttributedRangeError,
                           AttributedRangeError.outOfBoundsAttributedIndex(index: 50))
            XCTAssertEqual(error.localizedDescription,
                           "Provided attributed index is out of bounds (50)")
        }
    }
}
