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

/// Provides a SwiftUI displayable view for the composer UITextView component.
public struct WysiwygComposerView: UIViewRepresentable {
    // MARK: - Public

    @Binding public var focused: Bool

    // MARK: - Private

    private var viewModel: WysiwygComposerViewModelProtocol
    private var tintColor = Color.accentColor
    private var placeholderColor = Color(UIColor.placeholderText)
    private var placeholder: String?

    // MARK: - Public

    public init(focused: Binding<Bool>,
                viewModel: WysiwygComposerViewModelProtocol) {
        _focused = focused
        self.viewModel = viewModel
    }
    
    public func makeUIView(context: Context) -> PlaceholdableTextView {
        let textView = PlaceholdableTextView()
        
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
        textView.placeholderFont = UIFont.preferredFont(forTextStyle: .body)
        textView.placeholderColor = UIColor(placeholderColor)
        textView.placeholder = placeholder
        viewModel.textView = textView
        viewModel.updateCompressedHeightIfNeeded()
        return textView
    }

    public func updateUIView(_ uiView: PlaceholdableTextView, context: Context) {
        uiView.tintColor = UIColor(tintColor)
        uiView.placeholderColor = UIColor(placeholderColor)
        uiView.placeholder = placeholder
        
        switch (focused, uiView.isFirstResponder) {
        case (true, false): uiView.becomeFirstResponder()
        case (false, true): uiView.resignFirstResponder()
        default: break
        }
    }

    public func makeCoordinator() -> Coordinator {
        Coordinator($focused, viewModel.replaceText, viewModel.select, viewModel.didUpdateText)
    }

    /// Coordinates UIKit communication.
    public class Coordinator: NSObject, UITextViewDelegate, NSTextStorageDelegate {
        private var hasSkippedShouldAcceptChanges = true
        var focused: Binding<Bool>
        var replaceText: (NSRange, String) -> Bool
        var select: (NSRange) -> Void
        var didUpdateText: (Bool) -> Void
        init(_ focused: Binding<Bool>,
             _ replaceText: @escaping (NSRange, String) -> Bool,
             _ select: @escaping (NSRange) -> Void,
             _ didUpdateText: @escaping (Bool) -> Void) {
            self.focused = focused
            self.replaceText = replaceText
            self.select = select
            self.didUpdateText = didUpdateText
        }

        public func textView(_ textView: UITextView, shouldChangeTextIn range: NSRange, replacementText text: String) -> Bool {
            Logger.textView.logDebug(["Sel(att): \(range)",
                                      textView.logText,
                                      "Replacement: \"\(text)\""],
                                     functionName: #function)
            hasSkippedShouldAcceptChanges = false
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
            didUpdateText(!hasSkippedShouldAcceptChanges)
            hasSkippedShouldAcceptChanges = true
        }

        public func textViewDidChangeSelection(_ textView: UITextView) {
            Logger.textView.logDebug([textView.logSelection],
                                     functionName: #function)
            select(textView.selectedRange)
        }
        
        public func textViewDidBeginEditing(_ textView: UITextView) {
            focused.wrappedValue = true
        }
        
        public func textViewDidEndEditing(_ textView: UITextView) {
            focused.wrappedValue = false
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
