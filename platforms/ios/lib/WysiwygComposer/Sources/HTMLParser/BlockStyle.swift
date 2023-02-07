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

import UIKit

// MARK: - Public

/// Defines a custom background style for an attributed string element.
public struct BlockStyle: Equatable {
    /// Defines the padding to apply on a block element that is displayed with a custom style.
    public struct Padding: Equatable {
        /// Horizontal padding to apply.
        public let horizontal: CGFloat
        /// Vertical padding to apply.
        public let vertical: CGFloat

        /// Init.
        ///
        /// - Parameters:
        ///   - horizontal: Horizontal padding to apply.
        ///   - vertical: Vertical padding to apply.
        public init(horizontal: CGFloat, vertical: CGFloat) {
            self.horizontal = horizontal
            self.vertical = vertical
        }
    }

    /// Defines the type of rendering to apply on a block element that is displayed with a curstom style.
    public enum RenderingType: Equatable {
        /// Block is displayed with a background behind it.
        case background
        /// Block is displayed with a side element in its leading padding.
        case side(offset: CGFloat, width: CGFloat)
    }

    /// Init.
    ///
    /// - Parameters:
    ///   - backgroundColor: Background color of the element.
    ///   - borderColor: Border color of the  element.
    ///   - borderWidth: Border width of the element.
    ///   - cornerRadius: Corner radius of the element.
    ///   - padding: Padding of the element.
    ///   - type: Rendering type of the block.
    public init(backgroundColor: UIColor,
                borderColor: UIColor,
                borderWidth: CGFloat,
                cornerRadius: CGFloat,
                padding: Padding,
                type: RenderingType) {
        self.backgroundColor = backgroundColor
        self.borderColor = borderColor
        self.borderWidth = borderWidth
        self.cornerRadius = cornerRadius
        self.padding = padding
        self.type = type
    }

    /// Background color of the element.
    public let backgroundColor: UIColor
    /// Border color of the  element.
    public let borderColor: UIColor
    /// Border width of the element.
    public let borderWidth: CGFloat
    /// Corner radius of the element.
    public let cornerRadius: CGFloat
    /// Padding from the sides.
    public let padding: Padding
    /// Rendering type of the block.
    public let type: RenderingType
}

// MARK: - Internal

extension BlockStyle {
    /// Computes attributes applied on a `NSAttributedString` to display a block with this style.
    var attributes: [NSAttributedString.Key: Any] {
        [.blockStyle: self,
         .paragraphStyle: paragraphStyle]
    }

    /// Computes a default `NSParagraphStyle` with applied padding.
    var paragraphStyle: NSParagraphStyle {
        let paragraphStyle = NSParagraphStyle.default.mut()
        paragraphStyle.firstLineHeadIndent = padding.horizontal
        paragraphStyle.headIndent = padding.horizontal
        paragraphStyle.tailIndent = -padding.horizontal
        paragraphStyle.paragraphSpacingBefore = padding.vertical
        paragraphStyle.paragraphSpacing = padding.vertical
        return paragraphStyle
    }
}
