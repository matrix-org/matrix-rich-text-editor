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

extension CollectionDifference<Character> {
    /// Transforms the removals in the `CollectionDifference` into an array of ranges.
    var removedRanges: [NSRange] {
        removals.reduce([]) { partialResult, change in
            var partialRes = partialResult
            switch change {
            case .remove(offset: let offset, element: _, associatedWith: _):
                if let lastRange = partialRes.popLast() {
                    if lastRange.upperBound == offset {
                        partialRes.append(NSRange(location: lastRange.location, length: lastRange.length + 1))
                    } else {
                        partialRes.append(lastRange)
                        partialRes.append(NSRange(location: offset, length: 1))
                    }
                    return partialRes
                } else {
                    return [NSRange(location: offset, length: 1)]
                }
            default:
                return []
            }
        }
    }

    /// Transforms the insertions in the `CollectionDifference` into an array of ranges and associated text.
    var textInsertions: [(range: NSRange, text: String)] {
        insertions.reduce([]) { partialResult, change in
            var partialRes = partialResult
            switch change {
            case .insert(offset: let offset, element: let element, associatedWith: _):
                if let lastRange = partialRes.popLast() {
                    let (range, text) = lastRange
                    if range.upperBound == offset {
                        partialRes.append(
                            (NSRange(location: range.location, length: range.length + 1), text + String(element))
                        )
                    } else {
                        partialRes.append(lastRange)
                        partialRes.append((NSRange(location: offset, length: 1), String(element)))
                    }
                    return partialRes
                } else {
                    return [(NSRange(location: offset, length: 1), String(element))]
                }
            default:
                return []
            }
        }
    }
}
