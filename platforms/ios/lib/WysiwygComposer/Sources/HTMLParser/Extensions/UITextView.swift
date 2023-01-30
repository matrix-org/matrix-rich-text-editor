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
    func drawBackgroundStyleLayers() {
        layer
            .sublayers?[0]
            .sublayers?
            .compactMap { $0 as? BackgroundStyleLayer }
            .forEach { $0.removeFromSuperlayer() }

        attributedText.enumerateTypedAttribute(.blockStyle) { (style: BlockStyle, range: NSRange, _) in
            var styleLayer: BackgroundStyleLayer?
            switch style.type {
            case .background:
                let glyphRange = layoutManager.glyphRange(forCharacterRange: range, actualCharacterRange: nil)
                let rect = layoutManager
                    .boundingRect(forGlyphRange: glyphRange, in: self.textContainer)
                    .extendHorizontally(in: frame)

                styleLayer = BackgroundStyleLayer(style: style, frame: rect)
            case .side:
                let beginning = self.beginningOfDocument
                guard let start = self.position(from: beginning, offset: range.location) else { return }
                // removing the last character to avoid including the empty new line
                guard let end = self.position(from: start, offset: range.length - 1) else { return }
                guard let range = self.textRange(from: start, to: end) else { return }
                let rects = selectionRects(for: range)
                var textRect = CGRect.null
                for rect in rects {
                    textRect = textRect.union(rect.rect)
                }
                if !textRect.isNull {
                    let rect = CGRect(x: 5, y: textRect.origin.y, width: 4, height: textRect.size.height)
                    styleLayer = BackgroundStyleLayer(style: style, frame: rect)
                }
            }
            guard let styleLayer = styleLayer else { return }
            layer.sublayers?[0].insertSublayer(styleLayer, at: UInt32(layer.sublayers?.count ?? 0))
        }
    }
}

private final class BackgroundStyleLayer: CALayer {
    override init() {
        super.init()
    }

    init(style: BlockStyle, frame: CGRect) {
        super.init()

        self.frame = frame
        backgroundColor = style.backgroundColor.cgColor
        borderWidth = style.borderWidth
        borderColor = style.borderColor.cgColor
        cornerRadius = style.cornerRadius
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
    }
}
