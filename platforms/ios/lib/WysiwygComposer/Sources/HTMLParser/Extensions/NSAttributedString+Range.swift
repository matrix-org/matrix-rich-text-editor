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

    /// Compute an array of all detected occurences of discardable
    /// text (list prefixes, placeholder characters) within the given range
    /// of the attributed text.
    ///
    /// - Parameters:
    ///   - range: the range on which the elements should be detected. Entire range if omitted
    /// - Returns: an array of matching ranges
    func discardableTextRanges(in range: NSRange? = nil) -> [NSRange] {
        var ranges = [NSRange]()

        enumerateTypedAttribute(.discardableText, in: range) { (isDiscardable: Bool, range: NSRange, _) in
            if isDiscardable {
                ranges.append(range)
            }
        }

        return ranges
    }

    /// Compute an array of all parts of the attributed string that have been replaced
    /// with `PermalinkReplacer` usage within the given range. Also computes
    /// the offset between the replacement and the original part (i.e. if the original length
    /// is greater than the replacement range, this offset will be negative).
    ///
    /// - Parameter range: the range on which the elements should be detected. Entire range if omitted
    /// - Returns: an array of range and offsets.
    func replacementTextRanges(in range: NSRange? = nil) -> [(range: NSRange, offset: Int)] {
        var ranges = [(NSRange, Int)]()

        enumerateTypedAttribute(.originalLength) { (originalLength: Int, range: NSRange, _) in
            ranges.append((range, range.length - originalLength))
        }

        return ranges
    }

    /// Find occurences of parts of the attributed string that have been replaced
    /// within the range before given attributed index and compute the total offset
    /// that should be subtracted (HTML to attributed) or added (attributed to HTML)
    /// in order to compute the index properly.
    ///
    /// - Parameter attributedIndex: the index inside the attributed representation
    /// - Returns: Total offset of replacement ranges
    func replacementsOffsetAt(at attributedIndex: Int) -> Int {
        let range = NSRange(location: 0, length: attributedIndex)
        return replacementTextRanges(in: range)
            .map { range.contains($0.range) ? $0.offset : 0 }
            .reduce(0) { $0 - $1 }
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

        let replacementsOffset = replacementsOffsetAt(at: attributedIndex)
        return discardableTextRanges(in: .init(location: 0, length: attributedIndex))
            // All ranges length should be counted out, unless the last one end strictly after the
            // attributed index, in that case we only count out the difference (i.e. chars before the index)
            .map { $0.upperBound <= attributedIndex ? $0.length : attributedIndex - $0.location }
            .reduce(attributedIndex) { $0 - $1 } + replacementsOffset
    }

    /// Computes index inside the attributed representation from the index
    /// inside the HTML raw text.
    ///
    /// - Parameters:
    ///   - htmlIndex: the index inside the HTML raw text
    /// - Returns: the index inside the attributed representation
    func attributedPosition(at htmlIndex: Int) throws -> Int {
        var attributedIndex = try discardableTextRanges()
            // All ranges that have a HTML position before the provided index should be entirely counted.
            .filter { try htmlPosition(at: $0.location) <= htmlIndex }
            .reduce(htmlIndex) { $0 + $1.length }

        let replacementsOffset = replacementsOffsetAt(at: attributedIndex)
        attributedIndex -= replacementsOffset

        guard attributedIndex <= length else {
            throw AttributedRangeError
                .outOfBoundsHtmlIndex(index: htmlIndex)
        }

        return attributedIndex
    }
}

extension NSRange {
    func contains(_ otherRange: NSRange) -> Bool {
        contains(otherRange.location) && contains(otherRange.upperBound - 1)
    }
}
