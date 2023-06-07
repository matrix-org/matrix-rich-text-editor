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
import HTMLParser

/// Extension protocol for HTMLParser's `MentionReplacer` that handles replacement for markdown.
public protocol MentionReplacer: HTMLMentionReplacer {
    /// Called when the composer switches to plain text mode or when
    /// the client sets an HTML body as the current content of the composer
    /// in plain text mode. Provides the ability for the client to replace
    /// e.g. markdown links with a pillified representation.
    ///
    /// - Parameter attributedString: An attributed string containing the parsed markdown.
    /// - Returns: An attributed string with replaced content.
    func postProcessMarkdown(in attributedString: NSAttributedString) -> NSAttributedString

    /// Called when the composer switches out of plain text mode.
    /// Provides the ability for the client to restore a markdown-valid content
    /// for items altered using `postProcessMarkdown`.
    ///
    /// - Parameter attributedString: An attributed string containing the current content of the text view.
    /// - Returns: A valid markdown string.
    func restoreMarkdown(in attributedString: NSAttributedString) -> String
}
