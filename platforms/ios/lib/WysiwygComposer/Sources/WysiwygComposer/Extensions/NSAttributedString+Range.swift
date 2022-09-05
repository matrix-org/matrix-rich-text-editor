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
enum AttributedRangeError: LocalizedError, Equatable {
    /// Index is out of expected HTML bounds.
    case outOfBoundsHtmlIndex(index: Int)
    /// Index is out of attributed bounds.
    case outOfBoundsAttributedIndex(index: Int)

    var errorDescription: String? {
        switch self {
        case .outOfBoundsHtmlIndex(index: let index):
            return "Provided HTML index is out of expected bounds (\(index))"
        case .outOfBoundsAttributedIndex(index: let index):
            return "Provided attributed index is out of bounds (\(index))"
        }
    }
}

extension NSAttributedString {
    // MARK: - List prefixes detection
    /// Compute an array of all detected occurences of bulleted lists and
    /// numbered lists prefixes. ("1.", "•", ... with included tabulations and newline
    /// that are not represented in the HTML raw text).
    ///
    /// - Parameters:
    ///   - range: the range on which the elements should be detected. Entire range if omitted
    ///   - shouldIgnoreTrailingNewline: whether newline at the end of a list item should be ignored
    /// - Returns: an array of matching ranges
    func listPrefixesRanges(in range: NSRange? = nil,
                            shouldIgnoreTrailingNewline: Bool = true) -> [NSRange] {
        let numberedPrefixes = numberedListPrefixesRanges(in: range,
                                                          shouldIgnoreTrailingNewline: shouldIgnoreTrailingNewline)
        let bulletedPrefixes = bulletedListPrefixesRanges(in: range,
                                                          shouldIgnoreTrailingNewline: shouldIgnoreTrailingNewline)

        return (numberedPrefixes + bulletedPrefixes)
            .sorted(by: { $0.location < $1.location })
    }

    /// Compute an array of all detected occurences of bulleted lists
    /// prefixes. ("•" with included tabulations and newline
    /// that are not represented in the HTML raw text).
    ///
    /// - Parameters:
    ///   - range: the range on which the elements should be detected. Entire range if omitted
    ///   - shouldIgnoreTrailingNewline: whether newline at the end of a list item should be ignored
    /// - Returns: an array of matching ranges
    func bulletedListPrefixesRanges(in range: NSRange? = nil,
                                    shouldIgnoreTrailingNewline: Bool = true) -> [NSRange] {
        let pattern = shouldIgnoreTrailingNewline ? "[\\n]?\\t•\\t" : "\\t•\\t"
        let actualRange = range ?? .init(location: 0, length: length)
        // swiftlint:disable:next force_try
        let regex = try! NSRegularExpression(pattern: pattern)
        return regex
            .matches(in: string, range: actualRange)
            .compactMap { $0.range }
    }

    /// Compute an array of all detected occurences of numbered lists
    /// prefixes. ("1.", "2.", ... with included tabulations and newline
    /// that are not represented in the HTML raw text).
    ///
    /// - Parameters:
    ///   - range: the range on which the elements should be detected. Entire range if omitted
    ///   - shouldIgnoreTrailingNewline: whether newline at the end of a list item should be ignored
    /// - Returns: an array of matching ranges
    func numberedListPrefixesRanges(in range: NSRange? = nil,
                                    shouldIgnoreTrailingNewline: Bool = true) -> [NSRange] {
        let pattern = shouldIgnoreTrailingNewline ? "[\\n]?\\t\\d+\\.\\t" : "\\t\\d\\.\\t"
        let actualRange = range ?? .init(location: 0, length: length)
        // swiftlint:disable:next force_try
        let regex = try! NSRegularExpression(pattern: pattern)
        return regex
            .matches(in: string, range: actualRange)
            .compactMap { $0.range }
    }

    // MARK: - Indexes computation
    /// Computes index inside the HTML raw text from the index
    /// inside the attributed representation.
    ///
    /// - Parameters:
    ///   - attributedIndex: the index inside the attributed representation
    ///   - shouldIgnoreTrailingNewline: whether newline at the end of a list item should be ignored
    /// - Returns: the index inside the HTML raw text
    func htmlPosition(at attributedIndex: Int,
                      shouldIgnoreTrailingNewline: Bool = true) throws -> Int {
        guard attributedIndex <= length else {
            throw AttributedRangeError
                .outOfBoundsAttributedIndex(index: attributedIndex)
        }

        let prefixes = listPrefixesRanges(shouldIgnoreTrailingNewline: shouldIgnoreTrailingNewline)
        var actualIndex: Int = attributedIndex

        for listPrefix in prefixes {
            if listPrefix.upperBound <= attributedIndex {
                actualIndex -= listPrefix.length
            } else if listPrefix.contains(attributedIndex) && !character(at: attributedIndex).isNewline {
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
    ///   - shouldIgnoreTrailingNewline: whether newline at the end of a list item should be ignored
    /// - Returns: the index inside the attributed representation
    func attributedPosition(at htmlIndex: Int,
                            shouldIgnoreTrailingNewline: Bool = true) throws -> Int {
        let prefixes = listPrefixesRanges(shouldIgnoreTrailingNewline: shouldIgnoreTrailingNewline)
        var actualIndex: Int = htmlIndex

        for listPrefix in prefixes {
            if listPrefix.location < actualIndex {
                actualIndex += listPrefix.length
            } else if listPrefix.location == actualIndex {
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

    /// Computes range inside the HTML raw text from the
    /// range inside the attributed representation.
    ///
    /// - Parameters:
    ///   - attributedRange: the range inside the attributed representation
    ///   - shouldIgnoreTrailingNewline: whether newline at the end of a list item should be ignored
    /// - Returns: the range inside the HTML raw text
    func htmlRange(from attributedRange: NSRange,
                   shouldIgnoreTrailingNewline: Bool = true) throws -> NSRange {
        let start = try self.htmlPosition(at: attributedRange.location,
                                          shouldIgnoreTrailingNewline: shouldIgnoreTrailingNewline)
        let end = try self.htmlPosition(at: attributedRange.upperBound,
                                        shouldIgnoreTrailingNewline: shouldIgnoreTrailingNewline)
        return NSRange(location: start, length: end-start)
    }

    /// Computes a range inside the attributed representation from
    /// the range inside the HTML raw text.
    ///
    /// - Parameters:
    ///   - htmlRange: the range inside the HTML raw text
    ///   - shouldIgnoreTrailingNewline: whether newline at the end of a list item should be ignored
    /// - Returns: the range inside the attributed representation
    func attributedRange(from htmlRange: NSRange,
                         shouldIgnoreTrailingNewline: Bool = true) throws -> NSRange {
        let start = try self.attributedPosition(at: htmlRange.location,
                                                shouldIgnoreTrailingNewline: shouldIgnoreTrailingNewline)
        let end = try self.attributedPosition(at: htmlRange.upperBound,
                                              shouldIgnoreTrailingNewline: shouldIgnoreTrailingNewline)
        return NSRange(location: start, length: end-start)
    }
}

private extension Array where Element == NSRange {
    func containsIndex(_ index: Int) -> Bool {
        return self.contains { $0.location <= index && $0.upperBound >= index }
    }
}
