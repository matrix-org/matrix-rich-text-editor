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

/// Defines a custom background style for an attributed string element.
public struct BlockStyle: Equatable {
    public enum RenderingType: Equatable {
        case background
        case side(offset: CGFloat, width: CGFloat)
    }

    /// Init.
    ///
    /// - Parameters:
    ///   - backgroundColor: Background color of the element.
    ///   - borderColor: Border color of the  element.
    ///   - borderWidth: Border width of the element.
    ///   - cornerRadius: Corner radius of the element
    ///   - padding: Padding from the sides
    ///   - type: Rendering type of the block
    public init(backgroundColor: UIColor,
                borderColor: UIColor,
                borderWidth: CGFloat,
                cornerRadius: CGFloat,
                padding: CGFloat,
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
    /// Corner radius of the element
    public let cornerRadius: CGFloat
    /// Padding from the sides
    public let padding: CGFloat
    /// Rendering type of the block
    public let type: RenderingType
}
