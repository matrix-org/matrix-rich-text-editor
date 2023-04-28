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
    
    private static var defaultCSS: String {
        """
        blockquote {
            background-color: \(TempColor.quote.toHexString());
            display: block;
        }
        pre {
            background-color: \(TempColor.codeBlock.toHexString());
            display: block;
            font-family: monospace;
            white-space: pre;
            -coretext-fontname: Menlo-Regular;
            font-size: inherit;
        }
        code {
            background-color: \(TempColor.inlineCode.toHexString());
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
    
    private init() { }
    
    // MARK: - Public
    
    /// Parse given HTML to NSAttributedString with a standard style.
    ///
    /// - Parameters:
    ///   - html: HTML to parse
    ///   - encoding: String encoding to use
    ///   - style: Style to apply for HTML parsing
    ///   - permalinkReplacer:An object that might replace detected links.
    /// - Returns: An attributed string representation of the HTML content
    public static func parse(html: String,
                             encoding: String.Encoding = .utf16,
                             style: HTMLParserStyle = .standard,
                             permalinkReplacer: HTMLPermalinkReplacer? = nil) throws -> NSAttributedString {
        guard !html.isEmpty else {
            return NSAttributedString(string: "")
        }

        guard let data = html.data(using: encoding) else {
            throw BuildHtmlAttributedError.dataError(encoding: encoding)
        }
        
        let defaultFont = UIFont.preferredFont(forTextStyle: .body)
        
        let parsingOptions: [String: Any] = [
            DTUseiOS6Attributes: true,
            DTDefaultFontDescriptor: defaultFont.fontDescriptor,
            DTDefaultStyleSheet: DTCSSStylesheet(styleBlock: defaultCSS) as Any,
            DTDocumentPreserveTrailingSpaces: true,
        ]

        guard let builder = DTHTMLAttributedStringBuilder(html: data, options: parsingOptions, documentAttributes: nil) else {
            throw BuildHtmlAttributedError.dataError(encoding: encoding)
        }

        builder.willFlushCallback = { element in
            guard let element else { return }
            element.sanitize()
        }
        
        guard let attributedString = builder.generatedAttributedString() else {
            throw BuildHtmlAttributedError.dataError(encoding: encoding)
        }
        
        let mutableAttributedString = NSMutableAttributedString(attributedString: attributedString)
        mutableAttributedString.applyPostParsingCustomAttributes(style: style)

        if let permalinkReplacer {
            mutableAttributedString.replaceLinks(with: permalinkReplacer)
        }

        removeTrailingNewlineIfNeeded(from: mutableAttributedString, given: html)

        return mutableAttributedString
    }
    
    private static func removeTrailingNewlineIfNeeded(from mutableAttributedString: NSMutableAttributedString, given html: String) {
        // DTCoreText always adds a \n at the end of the document, which we need to remove
        // however it does not add it if </code> </a> are the last nodes.
        // It should give also issues with codeblock and blockquote when they contain newlines
        // but the usage of nbsp and zwsp solves that
        if mutableAttributedString.string.last == "\n",
           !html.hasSuffix("</code>"),
           !html.hasSuffix("</a>") {
            mutableAttributedString.deleteCharacters(
                in: NSRange(
                    location: mutableAttributedString.length - 1,
                    length: 1
                )
            )
        }
    }
}
