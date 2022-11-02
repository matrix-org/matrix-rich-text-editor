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

import UIKit

/// Provides tools to parse from HTML to NSAttributedString with a standard style.
final class HTMLParser {
    // MARK: - Private

    private init() { }

    // MARK: - Internal

    /// Parse given HTML to NSAttributedString with a standard style.
    ///
    /// - Parameters:
    ///   - html: HTML to parse
    ///   - encoding: string encoding to use
    ///   - textColor: text color to apply to the result string
    /// - Returns: an attributed string representation of the HTML content
    static func parse(html: String,
                      encoding: String.Encoding = .utf16,
                      textColor: UIColor) throws -> NSAttributedString {
        let htmlWithStyle = generateHtmlBodyWithStyle(htmlFragment: html)
        let attributed = try NSAttributedString(html: htmlWithStyle).changeColor(to: textColor)
        return attributed
    }
}

private extension HTMLParser {
    /// Generate an HTML body with standard style from given fragment.
    ///
    /// - Parameter htmlFragment: HTML fragment
    /// - Returns: HTML body
    static func generateHtmlBodyWithStyle(htmlFragment: String) -> String {
        "<html><head><style>body {font-family:-apple-system;font:-apple-system-body;}</style></head><body>\(htmlFragment)</body></html>"
    }
}
