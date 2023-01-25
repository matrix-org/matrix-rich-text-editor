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

    override public init(frame: CGRect, textContainer: NSTextContainer?) {
        super.init(frame: frame, textContainer: textContainer)
        contentMode = .redraw
    }
    
    required init?(coder: NSCoder) {
        super.init(coder: coder)
        contentMode = .redraw
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
}

private extension WysiwygTextView {
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
