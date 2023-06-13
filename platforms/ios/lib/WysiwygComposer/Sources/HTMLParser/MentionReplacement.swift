//
// Copyright 2023 The Matrix.org Foundation C.I.C
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

/// Represents a replacement in an instance of `NSAttributedString`
struct MentionReplacement {
    /// Range of the `NSAttributedString` where the replacement is located.
    let range: NSRange
    /// Data of the original content of the `NSAttributedString`.
    let content: MentionContent
}

// MARK: - Helpers

extension MentionReplacement {
    /// Computes the offset between the replacement and the original part (i.e. if the original length
    /// is greater than the replacement range, this offset will be negative).
    var offset: Int {
        range.length - content.rustLength
    }
}
