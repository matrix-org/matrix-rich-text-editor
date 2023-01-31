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
        codeBlockStyle: BlockStyle(backgroundColor: UIColor(red: 244 / 255, green: 246 / 255, blue: 250 / 255, alpha: 1.0),
                                   borderColor: UIColor(red: 227 / 255, green: 232 / 255, blue: 240 / 255, alpha: 1.0),
                                   borderWidth: 1.0,
                                   cornerRadius: 4.0,
                                   padding: 10,
                                   type: .background),
        quoteBlockStyle: BlockStyle(backgroundColor: UIColor(red: 244 / 255, green: 246 / 255, blue: 250 / 255, alpha: 1.0),
                                    borderColor: UIColor(red: 227 / 255, green: 232 / 255, blue: 240 / 255, alpha: 1.0),
                                    borderWidth: 0,
                                    cornerRadius: 0,
                                    padding: 25,
                                    type: .side(offset: 5, width: 4))
    )

    /// Color for standard text.
    public var textColor: UIColor
    /// Color for link text.
    public var linkColor: UIColor
    /// Code Block Style
    public var codeBlockStyle: BlockStyle
    /// Quote Block Style
    public var quoteBlockStyle: BlockStyle

    public init(textColor: UIColor,
                linkColor: UIColor,
                codeBlockStyle: BlockStyle,
                quoteBlockStyle: BlockStyle) {
        self.textColor = textColor
        self.linkColor = linkColor
        self.codeBlockStyle = codeBlockStyle
        self.quoteBlockStyle = quoteBlockStyle
    }
}
