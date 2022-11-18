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

/// Describes the output replacement from string diffing.
struct StringDifferReplacement: Equatable {
    let range: NSRange
    let text: String

    init(range: NSRange, text: String) {
        self.range = range
        self.text = text
    }

    init(location: Int, length: Int, text: String) {
        range = NSRange(location: location, length: length)
        self.text = text
    }
}

private struct StringDiff {
    let removals: UTF16Removals
    let insertions: UTF16Insertions

    var isEmpty: Bool {
        removals.isEmpty && insertions.isEmpty
    }
}

final class StringDiffer {
    // MARK: - Private

    private init() { }

    // MARK: - Internal

    static func replacement(from oldText: String, to newText: String) throws -> StringDifferReplacement? {
        let difference = newText.withNBSP.utf16Difference(from: oldText.withNBSP)

        guard !difference.isEmpty else {
            return nil
        }

        let removedRanges = difference.removals
        let textInsertions = difference.insertions

        guard removedRanges.count < 2, textInsertions.count < 2 else {
            throw StringDifferError.tooComplicated
        }

        if let removedRange = removedRanges.first {
            if let insertion = textInsertions.first {
                if insertion.range.location == removedRange.location {
                    // Replacement
                    return StringDifferReplacement(range: removedRange, text: insertion.text)
                } else {
                    throw StringDifferError.insertionsDontMatchRemovals
                }
            } else {
                // Simple removal
                return StringDifferReplacement(range: removedRange, text: "")
            }
        } else if let insertedRange = textInsertions.first {
            // Simple insertion
            return StringDifferReplacement(location: insertedRange.range.location, length: 0, text: insertedRange.text)
        } else {
            fatalError("Should never happen => difference is empty")
        }
    }
}

private extension String {
    /// Converts all whitespaces to NBSP to avoid diffs caused by HTML translations.
    var withNBSP: String {
        String(map { $0.isWhitespace ? Character("\u{00A0}") : $0 })
    }

    func utf16Difference(from otherString: String) -> StringDiff {
        let difference = difference(from: otherString)
        return StringDiff(removals: difference.utf16Removals(in: otherString),
                          insertions: difference.utf16Insertions(in: self))
    }
}
