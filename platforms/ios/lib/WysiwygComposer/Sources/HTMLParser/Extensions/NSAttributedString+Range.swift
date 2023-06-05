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

    /// Computes a string with all characters of the `NSAttributedString` that are actually part of the HTML.
    /// Positions in this string will return a range that conforms to the range returned by the Rust model.
    public var htmlChars: String {
        NSMutableAttributedString(attributedString: self)
            .removeDiscardableContent()
            .restoreReplacements()
            .string
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
    /// with `PermalinkReplacer` usage within the given range.
    ///
    /// - Parameter range: the range on which the elements should be detected. Entire range if omitted
    /// - Returns: an array of `Replacement`.
    func replacementTextRanges(in range: NSRange? = nil) -> [Replacement] {
        var replacements = [Replacement]()

        enumerateTypedAttribute(.originalContent) { (originalContent: OriginalContent, range: NSRange, _) in
            replacements.append(Replacement(range: range, originalContent: originalContent))
        }

        return replacements
    }

    /// Compute an array of all parts of the attributed string that have been replaced
    /// with `PermalinkReplacer` usage up to the provided index.
    ///
    /// - Parameter attributedIndex: the position until which the ranges should be computed.
    /// - Returns: an array of range and offsets.
    func replacementTextRanges(to attributedIndex: Int) -> [Replacement] {
        replacementTextRanges(in: .init(location: 0, length: attributedIndex))
    }

    /// Find occurences of parts of the attributed string that have been replaced
    /// within the range before given attributed index and compute the total offset
    /// that should be subtracted (HTML to attributed) or added (attributed to HTML)
    /// in order to compute the index properly.
    ///
    /// - Parameter attributedIndex: the index inside the attributed representation
    /// - Returns: Total offset of replacement ranges
    func replacementsOffsetAt(at attributedIndex: Int) -> Int {
        replacementTextRanges(to: attributedIndex)
            .compactMap { $0.range.upperBound <= attributedIndex ? Optional($0.offset) : nil }
            .reduce(0, -)
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
            .reduce(attributedIndex, -) + replacementsOffset
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
            .compactMap { try htmlPosition(at: $0.location) <= htmlIndex ? Optional($0.length) : nil }
            .reduce(htmlIndex, +)

        // Iterate replacement ranges in order and only account those
        // that are still in range after previous offset update.
        attributedIndex = replacementTextRanges(to: attributedIndex)
            .reduce(attributedIndex) { $1.range.location < $0 ? $0 + $1.offset : $0 }

        guard attributedIndex <= length else {
            throw AttributedRangeError
                .outOfBoundsHtmlIndex(index: htmlIndex)
        }

        return attributedIndex
    }
}

extension NSMutableAttributedString {
    /// Remove all discardable elements from the attributed
    /// string (i.e. list prefixes, zwsp placeholders, etc)
    ///
    /// - Returns: self (discardable)
    @discardableResult
    func removeDiscardableContent() -> Self {
        discardableTextRanges().reversed().forEach {
            replaceCharacters(in: $0, with: "")
        }

        return self
    }

    /// Restore original content from `Replacement` within the attributed string.
    ///
    /// - Returns: self (discardable)
    @discardableResult
    func restoreReplacements() -> Self {
        replacementTextRanges().reversed().forEach {
            replaceCharacters(in: $0.range, with: $0.originalContent.text)
        }

        return self
    }
}
