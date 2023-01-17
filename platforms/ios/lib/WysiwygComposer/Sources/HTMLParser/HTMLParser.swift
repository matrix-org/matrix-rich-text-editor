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

import DTCoreText
import UIKit

/// Provides tools to parse from HTML to NSAttributedString with a standard style.
public final class HTMLParser {
    // MARK: - Private

    private init() { }

    private static var codeBackgroundColor: UIColor!
    private static var quoteBackgroundColor: UIColor!

    // MARK: - Internal

    /// Parse given HTML to NSAttributedString with a standard style.
    ///
    /// - Parameters:
    ///   - html: HTML to parse
    ///   - encoding: string encoding to use
    ///   - textColor: text color to apply to the result string
    ///   - linkColor: text color to apply to the links
    ///   - codeBackgroundColor: color to apply to the background of code blocks
    /// - Returns: an attributed string representation of the HTML content
    public static func parse(html: String,
                             encoding: String.Encoding = .utf16,
                             textColor: UIColor,
                             linkColor: UIColor,
                             codeBackgroundColor: UIColor,
                             quoteBackgroundColor: UIColor) throws -> NSAttributedString {
        self.quoteBackgroundColor = quoteBackgroundColor
        self.codeBackgroundColor = codeBackgroundColor

        guard !html.isEmpty else {
            return NSAttributedString(string: "")
        }

        // Fixes additionnal unrequired "\n" inserted by DTCoreText
        var html = html
        html = "<span>" + html + "</span>"
        guard let data = html.data(using: .utf8) else {
            throw BuildHtmlAttributedError.dataError(encoding: encoding)
        }

        let defaultFont = UIFont.preferredFont(forTextStyle: .body)

        let parsingOptions: [String: Any] = [
            DTUseiOS6Attributes: true,
            DTDefaultFontDescriptor: defaultFont.fontDescriptor,
            DTDefaultStyleSheet: DTCSSStylesheet(styleBlock: defaultCSS) as Any,
        ]

        guard let builder = DTHTMLAttributedStringBuilder(html: data, options: parsingOptions, documentAttributes: nil) else {
            throw BuildHtmlAttributedError.dataError(encoding: encoding)
        }

        builder.willFlushCallback = { _ in
            // element?.sanitize(font: defaultFont)
        }

        guard let attributedString = builder.generatedAttributedString() else {
            throw BuildHtmlAttributedError.dataError(encoding: encoding)
        }

        let mutableAttributedString = NSMutableAttributedString(attributedString: attributedString)
        // removeDefaultForegroundColor(mutableAttributedString)
        // addLinks(mutableAttributedString)
        // removeLinkColors(mutableAttributedString)
        // replaceMarkedBlockquotes(mutableAttributedString)
        // replaceMarkedCodeBlocks(mutableAttributedString)
        // removeDTCoreTextArtifacts(mutableAttributedString)

        mutableAttributedString.addAttributes(
            [.foregroundColor: textColor], range: NSRange(location: 0, length: mutableAttributedString.length)
        )

        // This fixes an iOS bug where if some text is typed after a link, and then a whitespace is added the link color is overridden.
        mutableAttributedString.enumerateAttribute(.link, in: NSRange(location: 0, length: mutableAttributedString.length)) { value, range, _ in
            if value != nil {
                mutableAttributedString.addAttributes([.foregroundColor: linkColor], range: range)
            }
        }

        mutableAttributedString.applyQuoteBackgroundStyle()
        mutableAttributedString.applyCodeBlockBackgroundStyle()
        mutableAttributedString.applyInlineCodeBackgroundStyle(codeBackgroundColor: codeBackgroundColor)

        return mutableAttributedString
    }

    private static var defaultCSS: String {
        """
        blockquote {
            background-color: \(BackgroundStyle.quote.tempHexColor));
            display: block;
        }
        pre {
            background-color: \(BackgroundStyle.codeBlock.tempHexColor);
            display: inline;
            font-family: monospace;
            white-space: pre;
            -coretext-fontname: Menlo-Regular;
            font-size: inherit;
        }
        code {
            background-color: \(BackgroundStyle.inlineCode.tempHexColor);
            display: inline;
            font-family: monospace;
            white-space: pre;
            -coretext-fontname: Menlo-Regular;
            font-size: inherit;
        }
        h1,h2,h3 {
            font-size: 1.2em;
        }
        """
    }
}
