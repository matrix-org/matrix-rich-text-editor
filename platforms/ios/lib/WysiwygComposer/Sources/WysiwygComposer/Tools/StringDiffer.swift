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
import OSLog

/// Describe an error occurring during string diffing.
enum StringDifferError: LocalizedError, Equatable {
    case tooComplicated
    case insertionsDontMatchRemovals

    var errorDescription: String? {
        switch self {
        case .tooComplicated:
            return "Diff is too complicated to be handled as a simple replacement"
        case .insertionsDontMatchRemovals:
            return "Insertions don't match removals"
        }
    }
}

final class StringDiffer {
    // MARK: - Private

    private init() { }

    // MARK: - Internal

    static func replacement(from oldText: String, to newText: String) throws -> (range: NSRange, text: String)? {
        let difference = newText.withNBSP.difference(from: oldText.withNBSP)

        guard !difference.isEmpty else {
            return nil
        }

        let insertedText = String(difference.insertedText)

        // swiftlint:disable force_unwrapping
        switch (difference.removedRange, difference.insertedRange) {
        case (nil, nil):
            // Multiple insertions or removals (otherwise difference would have been empty)
            throw StringDifferError.tooComplicated
        case (nil, let insertedRange):
            // Just an insertion
            return (NSRange(location: insertedRange!.location, length: 0), insertedText)
        case (let removedRange, nil):
            if insertedText.isEmpty {
                // Just a removal
                return (removedRange!, insertedText)
            } else {
                // Inserted text spans over multiple ranges.
                throw StringDifferError.tooComplicated
            }
        case (let removedRange, let insertedRange):
            if removedRange!.location == insertedRange!.location {
                // Actual replacement of a piece of text with something else
                return (removedRange!, insertedText)
            } else {
                throw StringDifferError.insertionsDontMatchRemovals
            }
        }
        // swiftlint:enable force_unwrapping
    }
}

private extension String {
    /// Converts all whitespaces to NBSP to avoid diffs caused by HTML translations.
    var withNBSP: String {
        String(map { $0.isWhitespace ? Character("\u{00A0}") : $0 })
    }
}

private extension CollectionDifference<Character> {
    var removedRange: NSRange? {
        removals.reduce(nil) { partialResult, change in
            let index: Int
            switch change {
            case .remove(offset: let offset, element: _, associatedWith: _):
                index = offset
            default:
                return nil
            }

            if let partialResult = partialResult {
                if partialResult.upperBound == index {
                    return NSRange(location: partialResult.location, length: partialResult.length + 1)
                } else {
                    return nil
                }
            } else {
                return NSRange(location: index, length: 1)
            }
        }
    }

    var insertedRange: NSRange? {
        insertions.reduce(nil) { partialResult, change in
            let index: Int
            switch change {
            case .insert(offset: let offset, element: _, associatedWith: _):
                index = offset
            default:
                return nil
            }

            if let partialResult = partialResult {
                if partialResult.upperBound == index {
                    return NSRange(location: partialResult.location, length: partialResult.length + 1)
                } else {
                    return nil
                }
            } else {
                return NSRange(location: index, length: 1)
            }
        }
    }

    var insertedText: [Character] {
        compactMap {
            switch $0 {
            case .insert(offset: _, element: let element, associatedWith: _):
                return element
            default:
                return nil
            }
        }
    }
}
