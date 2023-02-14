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

import Foundation

/// Describe an eror occuring during HTML to attributed ranges manipulation.
public enum AttributedRangeError: LocalizedError, Equatable {
    /// Index is out of expected HTML bounds.
    case outOfBoundsHtmlIndex(index: Int)
    /// Index is out of attributed bounds.
    case outOfBoundsAttributedIndex(index: Int)

    public var errorDescription: String? {
        switch self {
        case let .outOfBoundsHtmlIndex(index: index):
            return "Provided HTML index is out of expected bounds (\(index))"
        case let .outOfBoundsAttributedIndex(index: index):
            return "Provided attributed index is out of bounds (\(index))"
        }
    }
}

extension NSAttributedString {
    // MARK: - Public

    /// Computes range inside the HTML raw text from the
    /// range inside the attributed representation.
    ///
    /// - Parameters:
    ///   - attributedRange: the range inside the attributed representation
    /// - Returns: the range inside the HTML raw text
    public func htmlRange(from attributedRange: NSRange) throws -> NSRange {
        let start = try htmlPosition(at: attributedRange.location)
        let end = try htmlPosition(at: attributedRange.upperBound)
        return NSRange(location: start, length: end - start)
    }

    /// Computes a range inside the attributed representation from
    /// the range inside the HTML raw text.
    ///
    /// - Parameters:
    ///   - htmlRange: the range inside the HTML raw text
    /// - Returns: the range inside the attributed representation
    public func attributedRange(from htmlRange: NSRange) throws -> NSRange {
        let start = try attributedPosition(at: htmlRange.location)
        let end = try attributedPosition(at: htmlRange.upperBound)
        return NSRange(location: start, length: end - start)
    }

    // MARK: - Internal

    /// Compute an array of all detected occurences of bulleted lists and
    /// numbered lists prefixes. ("1.", "•", ... with included tabulations and newline
    /// that are not represented in the HTML raw text).
    ///
    /// - Parameters:
    ///   - range: the range on which the elements should be detected. Entire range if omitted
    /// - Returns: an array of matching ranges
    func listPrefixesRanges(in range: NSRange? = nil) -> [NSRange] {
        let enumRange = range ?? .init(location: 0, length: length)
        var ranges = [NSRange]()

        enumerateAttribute(.DTField,
                           in: enumRange) { (attr: Any?, range: NSRange, _) in
            if attr != nil {
                ranges.append(range)
            }
        }

        return ranges
    }

    func discardableTextRanges(in range: NSRange? = nil) -> [NSRange] {
        let enumRange = range ?? .init(location: 0, length: length)
        var ranges = [NSRange]()

        enumerateAttribute(.discardableText,
                           in: enumRange) { (attr: Any?, range: NSRange, _) in
            if attr != nil {
                ranges.append(range)
            }
        }

        return ranges
    }

    /// Computes index inside the HTML raw text from the index
    /// inside the attributed representation.
    ///
    /// - Parameters:
    ///   - attributedIndex: the index inside the attributed representation
    /// - Returns: the index inside the HTML raw text
    func htmlPosition(at attributedIndex: Int) throws -> Int {
        guard attributedIndex <= length else {
            throw AttributedRangeError
                .outOfBoundsAttributedIndex(index: attributedIndex)
        }

        let discardableTextRanges = discardableTextRanges()
        var actualIndex = attributedIndex

        for discardableTextRange in discardableTextRanges where discardableTextRange.upperBound <= attributedIndex {
            actualIndex -= discardableTextRange.length
        }

        let prefixes = listPrefixesRanges()

        for listPrefix in prefixes {
            if listPrefix.upperBound <= attributedIndex {
                actualIndex -= listPrefix.length
            } else if listPrefix.contains(attributedIndex),
                      character(at: attributedIndex)?.isNewline == false {
                actualIndex -= (attributedIndex - listPrefix.location)
            }
        }

        return actualIndex
    }

    /// Computes index inside the attributed representation from the index
    /// inside the HTML raw text.
    ///
    /// - Parameters:
    ///   - htmlIndex: the index inside the HTML raw text
    /// - Returns: the index inside the attributed representation
    func attributedPosition(at htmlIndex: Int) throws -> Int {
        let discardableTextRanges = discardableTextRanges()
        var actualIndex = htmlIndex

        for discardableTextRange in discardableTextRanges where try htmlPosition(at: discardableTextRange.location) <= htmlIndex {
            actualIndex += discardableTextRange.length
        }

        let prefixes = listPrefixesRanges()

        for listPrefix in prefixes {
            let prefixLocation = try htmlPosition(at: listPrefix.location)
            if prefixLocation < htmlIndex {
                actualIndex += listPrefix.length
            } else if prefixLocation == htmlIndex {
                // Should only count the listPrefix in if we are
                // not on an inserted newline from end of <li>
                if !(attributedSubstring(from: .init(location: actualIndex, length: 1))
                    .string == "\n") {
                    actualIndex += listPrefix.length
                }
            }
        }

        guard actualIndex <= length else {
            throw AttributedRangeError
                .outOfBoundsHtmlIndex(index: htmlIndex)
        }

        return actualIndex
    }
}
