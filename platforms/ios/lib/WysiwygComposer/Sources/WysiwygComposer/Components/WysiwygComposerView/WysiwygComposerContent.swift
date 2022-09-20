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

@objcMembers
/// Defines message content displayed in the composer.
public class WysiwygComposerContent: NSObject {
    // MARK: - Public
    /// Displayed text, as plain text.
    public let plainText: String
    /// HTML representation of the displayed text.
    public let html: String
    /// Attributed string representation of the displayed text.
    public let attributed: NSAttributedString
    /// Range of the selected text within the attributed representation.
    public var attributedSelection: NSRange

    // MARK: - Internal
    /// Init.
    ///
    /// - Parameters:
    ///   - plainText: Displayed text, as plain text.
    ///   - html: HTML representation of the displayed text.
    ///   - attributed: Attributed string representation of the displayed text.
    ///   - attributedSelection: Range of the selected text within the attributed representation.
    init(plainText: String = "",
         html: String = "",
         attributed: NSAttributedString = .init(string: ""),
         attributedSelection: NSRange = .zero) {
        self.plainText = plainText
        self.html = html
        self.attributed = attributed
        self.attributedSelection = attributedSelection
    }
}
