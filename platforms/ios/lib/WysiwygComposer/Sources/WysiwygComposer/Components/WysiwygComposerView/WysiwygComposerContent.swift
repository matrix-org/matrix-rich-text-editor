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

/// Defines message content displayed in the composer.
@objcMembers
public class WysiwygComposerContent: NSObject {
    // MARK: - Public

    /// Markdown representation of the displayed text.
    public let markdown: String
    /// HTML representation of the displayed text.
    public let html: String

    // MARK: - Internal

    /// Init.
    ///
    /// - Parameters:
    ///   - markdown: Markdown representation of the displayed text.
    ///   - html: HTML representation of the displayed text.
    init(markdown: String = "",
         html: String = "") {
        self.markdown = markdown
        self.html = html
    }
}

public struct WysiwygComposerAttributedContent {
    /// Attributed string representation of the displayed text.
    public let text: NSAttributedString
    /// Range of the selected text within the attributed representation.
    public var selection: NSRange

    // MARK: - Internal

    /// Init.
    ///
    /// - Parameters:
    ///   - text: Attributed string representation of the displayed text.
    ///   - selection: Range of the selected text within the attributed representation.
    init(text: NSAttributedString = .init(string: ""),
         selection: NSRange = .zero) {
        self.text = text
        self.selection = selection
    }
}
