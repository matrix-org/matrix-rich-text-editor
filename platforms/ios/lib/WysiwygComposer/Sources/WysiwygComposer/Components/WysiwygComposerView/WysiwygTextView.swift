//
// Copyright 2022 The Matrix.org Foundation C.I.C
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

public class WysiwygTextView: UITextView {
    var shouldShowPlaceholder = true {
        didSet {
            setNeedsDisplay()
        }
    }
    
    var placeholder: String? {
        didSet {
            setNeedsDisplay()
        }
    }
    
    var placeholderColor: UIColor = .placeholderText {
        didSet {
            setNeedsDisplay()
        }
    }
    
    var placeholderFont = UIFont.preferredFont(forTextStyle: .body) {
        didSet {
            setNeedsDisplay()
        }
    }

    private let flusher = WysiwygPillsFlusher()

    override public init(frame: CGRect, textContainer: NSTextContainer?) {
        super.init(frame: frame, textContainer: textContainer)
        contentMode = .redraw
    }
    
    required init?(coder: NSCoder) {
        super.init(coder: coder)
        contentMode = .redraw
    }

    /// Register a pill view that has been added through `NSTextAttachmentViewProvider`.
    /// Should be called within the `loadView` function in order to clear the pills properly on text updates.
    ///
    /// - Parameter pillView: View to register.
    public func registerPillView(_ pillView: UIView) {
        flusher.registerPillView(pillView)
    }

    /// Apply given content to the text view. This will temporary disrupt the text view
    /// delegate in order to avoid having multiple unnecessary selection frowarded to
    /// the model. This is especially useful since setting the attributed text automatically
    /// moves the cursor to the end of the text and it might not be the expected behaviour.
    ///
    /// - Parameters:
    ///   - content: Content to apply.
    func apply(_ content: WysiwygComposerAttributedContent) {
        guard content.text != attributedText || content.selection != selectedRange else { return }

        performWithoutDelegate {
            self.attributedText = content.text
            // Set selection to {0, 0} then to expected position
            // avoids an issue with autocapitalization.
            self.selectedRange = .zero
            self.selectedRange = content.selection

            // Force redraw when applying content
            // FIXME: this could be improved further as we sometimes draw twice in a row.
            self.drawBackgroundStyleLayers()
        }
    }

    override public var attributedText: NSAttributedString! {
        willSet {
            flusher.flush()
        }
        didSet {
            delegate?.textViewDidChange?(self)
        }
    }
    
    override public func draw(_ rect: CGRect) {
        super.draw(rect)

        drawBackgroundStyleLayers()

        guard shouldShowPlaceholder, let placeholder = placeholder else {
            return
        }
        
        let attributes: [NSAttributedString.Key: Any] = [.foregroundColor: placeholderColor, .font: placeholderFont]
        
        let frame = rect.inset(by: .init(top: textContainerInset.top,
                                         left: textContainerInset.left + textContainer.lineFragmentPadding,
                                         bottom: textContainerInset.bottom,
                                         right: textContainerInset.right))
        
        placeholder.draw(in: frame, withAttributes: attributes)
    }

    override public func caretRect(for position: UITextPosition) -> CGRect {
        // Compute system expected caret rect.
        let rect = super.caretRect(for: position)
        // Determine rect for glyph at expected position.
        let index = offset(from: beginningOfDocument, to: position)
        let glyphRange = layoutManager.glyphRange(forCharacterRange: .init(location: index, length: 1), actualCharacterRange: nil)
        let glyphRect = layoutManager.boundingRect(forGlyphRange: glyphRange, in: textContainer)
        // Use the system caret rect for `x` position and width and correct
        // the `y` position and the height using the text glyphs.
        return CGRect(x: rect.minX,
                      y: glyphRect.minY - Constants.caretVerticalOffset,
                      width: rect.width,
                      height: glyphRect.height + 2 * Constants.caretVerticalOffset)
    }
}

private extension WysiwygTextView {
    enum Constants {
        /// Vertical offset applied at the top and the bottom of
        /// the caret to make it extend slightly from the text glyphs.
        static let caretVerticalOffset: CGFloat = 1.5
    }

    /// Perform an action while temporary removing the text view delegate.
    ///
    /// - Parameters:
    ///   - block: Code block to perform.
    func performWithoutDelegate(block: () -> Void) {
        let myDelegate = delegate
        delegate = nil
        block()
        delegate = myDelegate
    }
}
