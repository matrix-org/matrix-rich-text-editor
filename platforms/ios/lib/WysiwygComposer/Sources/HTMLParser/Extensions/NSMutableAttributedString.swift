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
    /// Sets the background style for detected quote within the attributed string.
    func applyQuoteBackgroundStyle() {
        enumerateTypedAttribute(.paragraphStyle) { (style: NSParagraphStyle, range: NSRange, _) in
            guard
                style.headIndent == 25,
                let newStyle = style.mutableCopy() as? NSMutableParagraphStyle else { return }

            newStyle.headIndent = 0
            newStyle.firstLineHeadIndent = 0
            newStyle.paragraphSpacing = 0
            addAttribute(.paragraphStyle, value: newStyle, range: range)
            addAttribute(.backgroundStyle, value: BackgroundStyle.quote, range: range)
            removeAttribute(.backgroundColor, range: range)
        }
    }

    /// Sets the background style for detected code blocks within the attributed string.
    func applyCodeBlockBackgroundStyle() {
        enumerateTypedAttribute(.font) { (font: UIFont, range: NSRange, _) in
            guard font.isMonospace,
                  backgroundColor(at: range.location).toHexString() == BackgroundStyle.codeBlock.tempHexColor else { return }

            addAttribute(.backgroundStyle, value: BackgroundStyle.codeBlock, range: range)
            removeAttribute(.backgroundColor, range: range)
        }
    }

    /// Sets the background style for detected inline code within the attributed string.
    ///
    /// - Parameters:
    ///   - codeBackgroundColor: the background color that should be applied to inline code
    func applyInlineCodeBackgroundStyle(codeBackgroundColor: UIColor) {
        enumerateTypedAttribute(.backgroundColor) { (color: UIColor, range: NSRange, _) in
            guard color.toHexString() == BackgroundStyle.inlineCode.tempHexColor else { return }

            // Note: for now inline code just uses standard NSAttributedString background color
            // to avoid issues where it spans accross multiple lines.
            addAttribute(.backgroundColor, value: codeBackgroundColor, range: range)
        }
    }
}
