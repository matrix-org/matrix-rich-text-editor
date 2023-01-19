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

public extension UITextView {
    /// Draw layers for all the HTML elements that require special background.
    ///
    /// - Parameters:
    ///   - style: Style for HTML parsing
    func drawBackgroundStyleLayers(style: HTMLParserStyle) {
        layer
            .sublayers?[0]
            .sublayers?
            .compactMap { $0 as? BackgroundStyleLayer }
            .forEach { $0.removeFromSuperlayer() }

        attributedText.enumerateTypedAttribute(.backgroundStyle) { (backgroundType: BackgroundType, range: NSRange, _) in
            guard backgroundType != .inlineCode else {
                // Not handled here
                return
            }

            let glyphRange = layoutManager.glyphRange(forCharacterRange: range, actualCharacterRange: nil)
            let rect = layoutManager
                .boundingRect(forGlyphRange: glyphRange, in: self.textContainer)
                .extendHorizontally(in: frame)

            let styleLayer = BackgroundStyleLayer(backgroundType: backgroundType, style: style, frame: rect)

            layer.sublayers?[0].insertSublayer(styleLayer, at: UInt32(layer.sublayers?.count ?? 0))
        }
    }
}

private final class BackgroundStyleLayer: CALayer {
    override init() {
        super.init()
    }

    init(backgroundType: BackgroundType, style: HTMLParserStyle, frame: CGRect) {
        super.init()

        self.frame = frame
        backgroundColor = backgroundType.backgroundColor(for: style).cgColor
        borderWidth = style.borderWidth
        borderColor = backgroundType.borderColor(for: style).cgColor
        cornerRadius = style.cornerRadius
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
    }
}

extension CGRect {
    func extendHorizontally(in frame: CGRect) -> CGRect {
        CGRect(x: frame.minX, y: minY, width: frame.width, height: height)
    }
}
