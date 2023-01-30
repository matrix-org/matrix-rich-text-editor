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
    /// Sets the background style for detected quote & code blocks within the attributed string.
    ///
    /// - Parameters:
    ///   - style: Style for HTML parsing.
    func applyBackgroundStyles(style: HTMLParserStyle) {
        enumerateTypedAttribute(.DTTextBlocks) { (value: NSArray, range: NSRange, _) in
            guard let textBlock = value.firstObject as? DTTextBlock else { return }
            switch textBlock.backgroundColor {
            case TempColor.codeBlock:
                addAttribute(.backgroundStyle, value: style.codeBlockBackgroundStyle, range: range)
                guard let paragraphStyle = NSMutableParagraphStyle.default.mutableCopy() as? NSMutableParagraphStyle else { return }
                paragraphStyle.firstLineHeadIndent = 10
                paragraphStyle.headIndent = 10
                paragraphStyle.tailIndent = -10
                addAttribute(.paragraphStyle, value: paragraphStyle, range: range)
            case TempColor.quote:
                addAttribute(.blockquote, value: style.quoteBackgroundColor, range: range)
                guard let paragraphStyle = NSMutableParagraphStyle.default.mutableCopy() as? NSMutableParagraphStyle else { return }
                paragraphStyle.firstLineHeadIndent = 25
                paragraphStyle.headIndent = 25
                paragraphStyle.tailIndent = -25
                addAttribute(.paragraphStyle, value: paragraphStyle, range: range)
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

    /// Finds any text that has been marked as discardable
    /// and either replaces it with ZWSP if contained overlaps with text marked with a background style
    /// or removes it otherwise
    func replaceOrDeleteDiscardableText() {
        enumerateTypedAttribute(.discardableText) { (discardable: Bool, range: NSRange, _) in
            guard discardable == true else { return }
            if self.attribute(.backgroundStyle, at: range.location, effectiveRange: nil) != nil {
                self.replaceCharacters(in: range, with: String.zwsp)
            } else {
                self.deleteCharacters(in: range)
            }
        }
    }

    /// Remove the vertical spacing for paragraphs in the entire attributed string.
    func removeParagraphVerticalSpacing() {
        enumerateTypedAttribute(.paragraphStyle) { (style: NSParagraphStyle, range: NSRange, _) in
            guard let mutableStyle = style.mutableCopy() as? NSMutableParagraphStyle else { return }

            mutableStyle.paragraphSpacing = 0
            mutableStyle.paragraphSpacingBefore = 0
            addAttribute(.paragraphStyle, value: mutableStyle as Any, range: range)
        }
    }
}
