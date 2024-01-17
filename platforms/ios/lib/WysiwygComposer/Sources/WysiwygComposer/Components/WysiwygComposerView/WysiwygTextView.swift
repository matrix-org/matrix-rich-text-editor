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

/// An internal delegate for the `WysiwygTextView`, used to bring paste and key commands events
/// to SwiftUI through the `WysiwygComposerView` coordinator class.
protocol WysiwygTextViewDelegate: AnyObject {
    /// Asks the delegate if the item attached to given item provider can be pasted into the application.
    ///
    /// - Parameter itemProvider: The item provider.
    /// - Returns: True if it can be pasted, false otherwise.
    func isPasteSupported(for itemProvider: NSItemProvider) -> Bool
    
    /// Notify the delegate that a paste event has beeb received by the text view.
    ///
    /// - Parameters:
    ///   - textView: Composer text view.
    ///   - provider: Item provider for the paste event.
    func textView(_ textView: UITextView, didReceivePasteWith provider: NSItemProvider)
    
    /// The supported key commands for the text view.
    var keyCommands: [WysiwygKeyCommand]? { get }
}

/// A markdown protocol used to provide additional context to the text view when displaying mentions through the text attachment provider
public protocol MentionDisplayHelper { }

public class WysiwygTextView: UITextView {
    private(set) var isDictationRunning = false
    
    /// Internal delegate for the text view.
    weak var wysiwygDelegate: WysiwygTextViewDelegate?
    
    private let flusher = WysiwygPillsFlusher()
    
    public var mentionDisplayHelper: MentionDisplayHelper?

    override public init(frame: CGRect, textContainer: NSTextContainer?) {
        super.init(frame: frame, textContainer: textContainer)
        commonInit()
    }
    
    required init?(coder: NSCoder) {
        super.init(coder: coder)
        commonInit()
    }
    
    private func commonInit() {
        contentMode = .redraw
        NotificationCenter.default.addObserver(self,
                                               selector: #selector(dictationDidStart),
                                               name: .init("UIKeyboardDidBeginDictationNotification"),
                                               object: nil)
    }
    
    @objc private func dictationDidStart(notification: Notification) {
        isDictationRunning = true
    }
    
    override public func dictationRecordingDidEnd() {
        // Do not call super, this is an optional objc protocol
        isDictationRunning = false
    }
    
    override public func dictationRecognitionFailed() {
        // Do not call super this is an optional objc protocol
        isDictationRunning = false
    }

    /// Register a pill view that has been added through `NSTextAttachmentViewProvider`.
    /// Should be called within the `loadView` function in order to clear the pills properly on text updates.
    ///
    /// - Parameter pillView: View to register.
    public func registerPillView(_ pillView: UIView) {
        flusher.registerPillView(pillView)
    }
    
    public func flushPills() {
        flusher.flush()
    }

    /// Apply given content to the text view. This will temporary disrupt the text view
    /// delegate in order to avoid having multiple unnecessary selection frowarded to
    /// the model. This is especially useful since setting the attributed text automatically
    /// moves the cursor to the end of the text and it might not be the expected behaviour.
    ///
    /// - Parameters:
    ///   - content: Content to apply.
    func apply(_ content: WysiwygComposerAttributedContent) {
        guard content.text.length == 0
            || content.text != attributedText
            || content.selection != selectedRange
        else { return }

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
            toggleAutocorrectionIfNeeded()
            delegate?.textViewDidChange?(self)
        }
    }
    
    override public func draw(_ rect: CGRect) {
        super.draw(rect)

        drawBackgroundStyleLayers()
    }

    override public func caretRect(for position: UITextPosition) -> CGRect {
        // Compute system expected caret rect.
        let rect = super.caretRect(for: position)
        // Use the system caret rect for `x` position and width and correct
        // the `y` position and the height.
        return CGRect(x: rect.minX,
                      y: rect.minY - Constants.caretVerticalOffset,
                      width: rect.width,
                      height: rect.height + 2 * Constants.caretVerticalOffset)
    }

    // Enter Key commands support

    override public var keyCommands: [UIKeyCommand]? {
        wysiwygDelegate?.keyCommands?.map { UIKeyCommand(input: $0.input,
                                                         modifierFlags: $0.modifierFlags,
                                                         action: #selector(keyCommandAction)) }
    }
    
    // This needs to be handled here, if the selector was directly added to the WysiwygKeyCommand it would not work properly.
    @objc private func keyCommandAction(sender: UIKeyCommand) {
        wysiwygDelegate?.keyCommands?.first(where: { $0.input == sender.input && $0.modifierFlags == sender.modifierFlags })?.action()
    }

    // Paste support

    override public func canPerformAction(_ action: Selector, withSender sender: Any?) -> Bool {
        guard !super.canPerformAction(action, withSender: sender) else {
            return true
        }

        guard action == #selector(paste(_:)),
              let itemProvider = UIPasteboard.general.itemProviders.first,
              let wysiwygDelegate else {
            return false
        }

        return wysiwygDelegate.isPasteSupported(for: itemProvider)
    }

    override public func paste(_ sender: Any?) {
        guard let provider = UIPasteboard.general.itemProviders.first,
              let wysiwygDelegate,
              wysiwygDelegate.isPasteSupported(for: provider) else {
            // If the item is not supported by the hosting application
            // just try pasting its contents into the textfield
            super.paste(sender)
            return
        }

        wysiwygDelegate.textView(self, didReceivePasteWith: provider)
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
