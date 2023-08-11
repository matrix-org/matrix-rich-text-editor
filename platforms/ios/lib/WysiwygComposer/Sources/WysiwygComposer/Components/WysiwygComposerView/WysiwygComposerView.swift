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

import OSLog
import SwiftUI

/// A protocol to implement in order to inform the composer if a specific item
/// can be pasted into the application.
public protocol WysiwygItemProviderHelper {
    /// Determine if the item attached to given item provider can be pasted into the application.
    ///
    /// - Parameter itemProvider: The item provider.
    /// - Returns: True if it can be pasted, false otherwise.
    func isPasteSupported(for itemProvider: NSItemProvider) -> Bool
}

/// Handler for key commands.
public typealias KeyCommandHandler = (WysiwygKeyCommand) -> Bool
/// Handler for paste events.
public typealias PasteHandler = (NSItemProvider) -> Void

/// Provides a SwiftUI displayable view for the composer UITextView component.
public struct WysiwygComposerView: UIViewRepresentable {
    // MARK: - Public

    /// A helper to determine if an item can be pasted into the hosting application.
    /// If omitted, most non-text paste events will be ignored.
    let itemProviderHelper: WysiwygItemProviderHelper?
    /// A handler for key commands. If omitted, default behaviour will be applied. See `WysiwygKeyCommand.swift`.
    let keyCommandHandler: KeyCommandHandler?
    /// A handler for paste events. If omitted, the composer will try to paste content as raw text.
    let pasteHandler: PasteHandler?

    // MARK: - Private

    private var viewModel: WysiwygComposerViewModelProtocol
    private var tintColor = Color.accentColor
    private var placeholderColor = Color(UIColor.placeholderText)
    private var placeholder: String?

    // MARK: - Public

    public init(viewModel: WysiwygComposerViewModelProtocol,
                itemProviderHelper: WysiwygItemProviderHelper?,
                keyCommandHandler: KeyCommandHandler?,
                pasteHandler: PasteHandler?) {
        self.itemProviderHelper = itemProviderHelper
        self.keyCommandHandler = keyCommandHandler
        self.pasteHandler = pasteHandler
        self.viewModel = viewModel
    }
    
    public func makeUIView(context: Context) -> WysiwygTextView {
        let textView = viewModel.textView
        
        textView.accessibilityIdentifier = "WysiwygComposer"
        textView.font = UIFont.preferredFont(forTextStyle: .body)
        textView.autocapitalizationType = .sentences
        textView.isSelectable = true
        textView.isUserInteractionEnabled = true
        textView.delegate = context.coordinator
        textView.textStorage.delegate = context.coordinator
        textView.textContainerInset = .zero
        textView.textContainer.lineFragmentPadding = 0
        textView.adjustsFontForContentSizeCategory = true
        textView.backgroundColor = .clear
        textView.tintColor = UIColor(tintColor)
        textView.clipsToBounds = false
        textView.wysiwygDelegate = context.coordinator
        textView.placeholderFont = UIFont.preferredFont(forTextStyle: .body)
        textView.placeholderColor = UIColor(placeholderColor)
        textView.placeholder = placeholder
        viewModel.updateCompressedHeightIfNeeded()
        return textView
    }

    public func updateUIView(_ uiView: WysiwygTextView, context: Context) {
        uiView.tintColor = UIColor(tintColor)
        uiView.placeholderColor = UIColor(placeholderColor)
        uiView.placeholder = placeholder
    }

    public func makeCoordinator() -> Coordinator {
        Coordinator(viewModel.replaceText,
                    viewModel.select,
                    viewModel.didUpdateText,
                    viewModel.enter,
                    itemProviderHelper: itemProviderHelper,
                    keyCommandHandler: keyCommandHandler,
                    pasteHandler: pasteHandler)
    }

    /// Coordinates UIKit communication.
    public class Coordinator: NSObject, UITextViewDelegate, NSTextStorageDelegate, WysiwygTextViewDelegate {
        var replaceText: (NSRange, String) -> Bool
        var select: (NSRange) -> Void
        var didUpdateText: () -> Void
        var enter: () -> Void

        private let itemProviderHelper: WysiwygItemProviderHelper?
        private let keyCommandHandler: KeyCommandHandler?
        private let pasteHandler: PasteHandler?

        init(_ replaceText: @escaping (NSRange, String) -> Bool,
             _ select: @escaping (NSRange) -> Void,
             _ didUpdateText: @escaping () -> Void,
             _ enter: @escaping () -> Void,
             itemProviderHelper: WysiwygItemProviderHelper?,
             keyCommandHandler: KeyCommandHandler?,
             pasteHandler: PasteHandler?) {
            self.replaceText = replaceText
            self.select = select
            self.didUpdateText = didUpdateText
            self.enter = enter
            self.itemProviderHelper = itemProviderHelper
            self.keyCommandHandler = keyCommandHandler
            self.pasteHandler = pasteHandler
        }

        public func textView(_ textView: UITextView, shouldChangeTextIn range: NSRange, replacementText text: String) -> Bool {
            Logger.textView.logDebug(["Sel(att): \(range)",
                                      textView.logText,
                                      "Replacement: \"\(text)\""],
                                     functionName: #function)
            return replaceText(range, text)
        }
        
        public func textViewDidChange(_ textView: UITextView) {
            Logger.textView.logDebug(
                [
                    textView.logSelection,
                    textView.logText,
                ],
                functionName: #function
            )
            didUpdateText()
            textView.toggleAutocorrectionIfNeeded()
        }

        public func textViewDidChangeSelection(_ textView: UITextView) {
            Logger.textView.logDebug([textView.logSelection],
                                     functionName: #function)
            select(textView.selectedRange)
        }
        
        public func textView(_ textView: UITextView,
                             shouldInteractWith URL: URL,
                             in characterRange: NSRange,
                             interaction: UITextItemInteraction) -> Bool {
            guard interaction == .invokeDefaultAction else {
                return true
            }
            textView.selectedRange = characterRange
            return false
        }

        func isPasteSupported(for itemProvider: NSItemProvider) -> Bool {
            guard let itemProviderHelper else {
                return false
            }

            return itemProviderHelper.isPasteSupported(for: itemProvider)
        }

        func textViewDidReceiveKeyCommand(_ textView: UITextView, keyCommand: WysiwygKeyCommand) {
            if !handleKeyCommand(keyCommand) {
                processDefault(for: keyCommand)
            }
        }

        func textView(_ textView: UITextView, didReceivePasteWith provider: NSItemProvider) {
            pasteHandler?(provider)
        }

        private func handleKeyCommand(_ keyCommand: WysiwygKeyCommand) -> Bool {
            guard let keyCommandHandler else { return false }

            return keyCommandHandler(keyCommand)
        }

        private func processDefault(for keyCommand: WysiwygKeyCommand) {
            switch keyCommand {
            case .enter, .shiftEnter:
                enter()
            }
        }
    }
}

public extension WysiwygComposerView {
    /// Sets the tintColor of the rich text editor textView, if not used the default value is Color.accent.
    func tintColor(_ tintColor: Color) -> Self {
        var newSelf = self
        newSelf.tintColor = tintColor
        return newSelf
    }
    
    /// Apply a placeholder text to the composer
    ///
    /// - Parameters:
    ///   - placeholder: The placeholder text to display, if nil, no placeholder is displayed.
    ///   - color: The color of the placeholder text when displayed, default value is Color(UIColor.placeholderText).
    func placeholder(_ placeholder: String?, color: Color = Color(UIColor.placeholderText)) -> Self {
        var newSelf = self
        newSelf.placeholder = placeholder
        newSelf.placeholderColor = color
        return newSelf
    }
}

// MARK: - Logger

private extension Logger {
    static let textView = Logger(subsystem: subsystem, category: "TextView")
}
