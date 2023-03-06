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

import DTCoreText
import Foundation

extension NSAttributedString.Key {
    // MARK: - DTCoreText Internal Keys

    static let DTTextBlocks: NSAttributedString.Key = .init(rawValue: DTTextBlocksAttribute)
    static let DTField: NSAttributedString.Key = .init(rawValue: DTFieldAttribute)
    static let DTTextLists: NSAttributedString.Key = .init(rawValue: DTTextListsAttribute)

    // MARK: - Custom Keys

    /// Attribute for parts of the string that require some custom drawing (e.g. code blocks, quotes).
    static let blockStyle: NSAttributedString.Key = .init(rawValue: "BlockStyleAttributeKey")
    /// Attribute for parts of the string that should be removed for HTML selection computation.
    /// Should include both placeholder characters such as NBSP and ZWSP, as well as list prefixes.
    static let discardableText: NSAttributedString.Key = .init(rawValue: "DiscardableAttributeKey")
    /// Attributes for original length of a replaced element. This should be added anytime a part of the attributed string
    /// is replaced, in order for the composer to compute the expected HTML/attributed range properly.
    static let originalLength: NSAttributedString.Key = .init(rawValue: "OriginalLengthAttributeKey")
}
