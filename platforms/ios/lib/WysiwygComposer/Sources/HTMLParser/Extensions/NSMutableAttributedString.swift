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
import UIKit

extension NSMutableAttributedString {
    /// Apply all custom attributes expected by the composer's `UITextView` on self.
    ///
    /// - Parameters:
    ///   - style: Style for HTML parsing.
    func applyPostParsingCustomAttributes(style: HTMLParserStyle) {
        addAttributes(
            [.foregroundColor: style.textColor], range: NSRange(location: 0, length: length)
        )

        // This fixes an iOS bug where if some text is typed after a link, and then a whitespace is added the link color is overridden.
        enumerateTypedAttribute(.link) { (_: URL, range: NSRange, _) in
            removeAttribute(.underlineStyle, range: range)
            removeAttribute(.underlineColor, range: range)
            addAttributes([.foregroundColor: style.linkColor], range: range)
        }

        removeParagraphVerticalSpacing()
        applyBackgroundStyles(style: style)
        applyInlineCodeBackgroundStyle(codeBackgroundColor: style.codeBlockStyle.backgroundColor)
        replacePlaceholderTexts()
        applyDiscardableToListPrefixes()
    }

    /// Replace parts of the attributed string that represents links by
    /// a new attributed string part provided by the hosting app `HTMLPermalinkReplacer`.
    ///
    /// - Parameter permalinkReplacer: The permalink replacer providing new attributed strings.
    func replaceLinks(with permalinkReplacer: HTMLPermalinkReplacer) {
        enumerateTypedAttribute(.link) { (url: URL, range: NSRange, _) in
            if let replacement = permalinkReplacer.replacementForLink(
                url.absoluteString,
                text: self.mutableString.substring(with: range)
            ) {
                let originalText = self.attributedSubstring(from: range).string
                self.replaceCharacters(in: range, with: replacement)
                self.addAttribute(.originalContent,
                                  value: OriginalContent(text: originalText),
                                  range: .init(location: range.location, length: replacement.length))
            }
        }
    }
}

private extension NSMutableAttributedString {
    /// Remove the vertical spacing for paragraphs in the entire attributed string.
    func removeParagraphVerticalSpacing() {
        enumerateTypedAttribute(.paragraphStyle) { (style: NSParagraphStyle, range: NSRange, _) in
            let mutableStyle = style.mut()
            mutableStyle.paragraphSpacing = 0
            mutableStyle.paragraphSpacingBefore = 0
            addAttribute(.paragraphStyle, value: mutableStyle as Any, range: range)
        }
    }

    /// Sets the background style for detected quote & code blocks within the attributed string.
    ///
    /// - Parameters:
    ///   - style: Style for HTML parsing.
    func applyBackgroundStyles(style: HTMLParserStyle) {
        enumerateTypedAttribute(.DTTextBlocks) { (value: NSArray, range: NSRange, _) in
            guard let textBlock = value.firstObject as? DTTextBlock else { return }
            switch textBlock.backgroundColor {
            case TempColor.codeBlock:
                addAttributes(style.codeBlockStyle.attributes, range: range)
                mutableString.replaceOccurrences(of: String.carriageReturn, with: String.lineSeparator, range: range.excludingLast)
                // Remove inline code background color, if it exists.
                removeAttribute(.backgroundColor, range: range)
            case TempColor.quote:
                addAttributes(style.quoteBlockStyle.attributes, range: range)
                mutableString.replaceOccurrences(of: String.lineFeed, with: String.lineSeparator, range: range.excludingLast)
            default:
                break
            }
        }
    }

    /// Sets the background style for detected inline code within the attributed string.
    ///
    /// - Parameters:
    ///   - codeBackgroundColor: the background color that should be applied to inline code
    func applyInlineCodeBackgroundStyle(codeBackgroundColor: UIColor) {
        enumerateTypedAttribute(.backgroundColor) { (color: UIColor, range: NSRange, _) in
            guard color == TempColor.inlineCode else { return }

            // Note: for now inline code just uses standard NSAttributedString background color
            // to avoid issues where it spans accross multiple lines.
            addAttribute(.backgroundColor, value: codeBackgroundColor, range: range)
        }
    }

    /// Finds any text that has been marked as discardable and replaces it with ZWSP
    func replacePlaceholderTexts() {
        enumerateTypedAttribute(.discardableText) { (discardable: Bool, range: NSRange, _) in
            guard discardable == true else { return }
            self.replaceCharacters(in: range, with: String.zwsp)
        }
    }

    /// Finds any list prefix field inside the string and mark them as discardable text.
    func applyDiscardableToListPrefixes() {
        enumerateTypedAttribute(.DTField) { (_: String, range: NSRange, _) in
            addAttribute(.discardableText, value: true, range: range)
        }
    }
}
