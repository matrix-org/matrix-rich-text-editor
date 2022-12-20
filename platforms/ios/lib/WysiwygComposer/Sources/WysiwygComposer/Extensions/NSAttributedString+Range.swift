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
        case let .outOfBoundsHtmlIndex(index: index):
            return "Provided HTML index is out of expected bounds (\(index))"
        case let .outOfBoundsAttributedIndex(index: index):
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
    /// - Returns: an array of matching ranges
    func listPrefixesRanges(in range: NSRange? = nil) -> [NSRange] {
        let numberedPrefixes = numberedListPrefixesRanges(in: range)
        let bulletedPrefixes = bulletedListPrefixesRanges(in: range)

        return (numberedPrefixes + bulletedPrefixes)
            .sorted(by: { $0.location < $1.location })
    }

    /// Compute an array of all detected occurences of bulleted lists
    /// prefixes. ("•" with included tabulations and newline
    /// that are not represented in the HTML raw text).
    ///
    /// - Parameters:
    ///   - range: the range on which the elements should be detected. Entire range if omitted
    /// - Returns: an array of matching ranges
    func bulletedListPrefixesRanges(in range: NSRange? = nil) -> [NSRange] {
        let pattern = "\\t•\\t"
        let actualRange = range ?? .init(location: 0, length: length)
        // swiftlint:disable:next force_try
        let regex = try! NSRegularExpression(pattern: pattern)
        return regex
            .matches(in: string, range: actualRange)
            .map(\.range)
    }

    /// Compute an array of all detected occurences of numbered lists
    /// prefixes. ("1.", "2.", ... with included tabulations and newline
    /// that are not represented in the HTML raw text).
    ///
    /// - Parameters:
    ///   - range: the range on which the elements should be detected. Entire range if omitted
    /// - Returns: an array of matching ranges
    func numberedListPrefixesRanges(in range: NSRange? = nil) -> [NSRange] {
        let pattern = "\\t\\d+\\.\\t"
        let actualRange = range ?? .init(location: 0, length: length)
        // swiftlint:disable:next force_try
        let regex = try! NSRegularExpression(pattern: pattern)
        return regex
            .matches(in: string, range: actualRange)
            .map(\.range)
    }

    // MARK: - Indexes computation

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

        let prefixes = listPrefixesRanges()
        var actualIndex: Int = attributedIndex
        
        guard actualIndex > 0 else {
            return actualIndex
        }
        
        if !prefixes.isEmpty {
            actualIndex += 1
        }

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
        let prefixes = listPrefixesRanges()
        var actualIndex: Int = htmlIndex
        
        guard actualIndex > 0 else {
            return actualIndex
        }
        
        if !prefixes.isEmpty {
            actualIndex -= 1
        }
        
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
    /// - Returns: the range inside the HTML raw text
    func htmlRange(from attributedRange: NSRange) throws -> NSRange {
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
    func attributedRange(from htmlRange: NSRange) throws -> NSRange {
        let start = try attributedPosition(at: htmlRange.location)
        let end = try attributedPosition(at: htmlRange.upperBound)
        return NSRange(location: start, length: end - start)
    }
}

private extension Array where Element == NSRange {
    func containsIndex(_ index: Int) -> Bool {
        contains { $0.location <= index && $0.upperBound >= index }
    }
}
