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
            case TempColor.quote:
                addAttribute(.backgroundStyle, value: style.quoteBackgroundStyle, range: range)
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
}
