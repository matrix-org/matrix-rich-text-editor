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
import UIKit

/// Describe style for the HTML parser.
public struct HTMLParserStyle {
    // MARK: - Public

    public static let standard = HTMLParserStyle(
        textColor: UIColor.label,
        linkColor: UIColor.link,
        codeBackgroundColor: UIColor(red: 244 / 255, green: 246 / 255, blue: 250 / 255, alpha: 1.0),
        codeBorderColor: UIColor(red: 227 / 255, green: 232 / 255, blue: 240 / 255, alpha: 1.0),
        quoteBackgroundColor: UIColor(red: 244 / 255, green: 246 / 255, blue: 250 / 255, alpha: 1.0),
        quoteBorderColor: UIColor(red: 227 / 255, green: 232 / 255, blue: 240 / 255, alpha: 1.0),
        borderWidth: 1.0,
        cornerRadius: 4.0
    )

    /// Color for standard text.
    public var textColor: UIColor
    /// Color for link text.
    public var linkColor: UIColor
    /// Background color for code blocks / inline code.
    public var codeBackgroundColor: UIColor
    /// Border color for code blocks / inline code.
    public var codeBorderColor: UIColor
    /// Background color for quotes.
    public var quoteBackgroundColor: UIColor
    /// Border color for quotes.
    public var quoteBorderColor: UIColor
    /// Border width for custom backgrounds.
    public var borderWidth: CGFloat
    /// Corner radius for custom backgrounds.
    public var cornerRadius: CGFloat

    // MARK: - Internal

    var codeBlockBackgroundStyle: BackgroundStyle {
        BackgroundStyle(backgroundColor: codeBackgroundColor,
                        borderColor: codeBorderColor,
                        borderWidth: borderWidth,
                        cornerRadius: cornerRadius)
    }

    var quoteBackgroundStyle: BackgroundStyle {
        BackgroundStyle(backgroundColor: quoteBackgroundColor,
                        borderColor: quoteBorderColor,
                        borderWidth: borderWidth,
                        cornerRadius: cornerRadius)
    }

    public init(textColor: UIColor,
                linkColor: UIColor,
                codeBackgroundColor: UIColor,
                codeBorderColor: UIColor,
                quoteBackgroundColor: UIColor,
                quoteBorderColor: UIColor,
                borderWidth: CGFloat,
                cornerRadius: CGFloat) {
        self.textColor = textColor
        self.linkColor = linkColor
        self.codeBackgroundColor = codeBackgroundColor
        self.codeBorderColor = codeBorderColor
        self.quoteBackgroundColor = quoteBackgroundColor
        self.quoteBorderColor = quoteBorderColor
        self.borderWidth = borderWidth
        self.cornerRadius = cornerRadius
    }
}
