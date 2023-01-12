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

public extension NSRange {
    /// Returns a range at starting location, i.e. {0, 0}.
    static let zero = Self(location: 0, length: 0)

    /// Init a range that is located between the two provided ranges.
    /// Fails if the second range starts before the end of the first range.
    ///
    /// - Parameters:
    ///   - range1: range before the range to create
    ///   - range2: range after the range to create
    init?(between range1: NSRange, and range2: NSRange) {
        guard range1.upperBound <= range2.location else { return nil }

        self.init(location: range1.upperBound, length: range2.location - range1.upperBound)
    }
}
