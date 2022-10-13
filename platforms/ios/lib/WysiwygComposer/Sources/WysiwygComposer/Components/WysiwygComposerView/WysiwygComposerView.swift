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
    // MARK: - Internal

    public var content: WysiwygComposerContent
    public var replaceText: (NSAttributedString, NSRange, String) -> Bool
    public var select: (NSAttributedString, NSRange) -> Void
    public var didUpdateText: (UITextView) -> Void
    
    private var tintColor = Color.accentColor
    
    public init(content: WysiwygComposerContent,
                replaceText: @escaping (NSAttributedString, NSRange, String) -> Bool,
                select: @escaping (NSAttributedString, NSRange) -> Void,
                didUpdateText: @escaping (UITextView) -> Void) {
        self.content = content
        self.replaceText = replaceText
        self.select = select
        self.didUpdateText = didUpdateText
    }
    
    public func makeUIView(context: Context) -> UITextView {
        let textView = UITextView()

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
        return textView
    }

    public func updateUIView(_ uiView: UITextView, context: Context) {
        Logger.textView.logDebug([content.logAttributedSelection,
                                  content.logText],
                                 functionName: #function)
        uiView.apply(content)
        context.coordinator.didUpdateText(uiView)
        uiView.tintColor = UIColor(tintColor)
    }

    public func makeCoordinator() -> Coordinator {
        Coordinator(replaceText, select, didUpdateText)
    }

    /// Coordinates UIKit communication.
    public class Coordinator: NSObject, UITextViewDelegate, NSTextStorageDelegate {
        var replaceText: (NSAttributedString, NSRange, String) -> Bool
        var select: (NSAttributedString, NSRange) -> Void
        var didUpdateText: (UITextView) -> Void

        init(_ replaceText: @escaping (NSAttributedString, NSRange, String) -> Bool,
             _ select: @escaping (NSAttributedString, NSRange) -> Void,
             _ didUpdateText: @escaping (UITextView) -> Void) {
            self.replaceText = replaceText
            self.select = select
            self.didUpdateText = didUpdateText
        }

        public func textView(_ textView: UITextView, shouldChangeTextIn range: NSRange, replacementText text: String) -> Bool {
            Logger.textView.logDebug(["Sel(att): \(range)",
                                      textView.logText,
                                      "Replacement: \"\(text)\""],
                                     functionName: #function)
            return replaceText(textView.attributedText, range, text)
        }

        public func textViewDidChange(_ textView: UITextView) {
            Logger.textView.logDebug([textView.logSelection,
                                      textView.logText],
                                     functionName: #function)
            didUpdateText(textView)
        }

        public func textViewDidChangeSelection(_ textView: UITextView) {
            Logger.textView.logDebug([textView.logSelection],
                                     functionName: #function)
            select(textView.attributedText, textView.selectedRange)
        }
    }
}

public extension WysiwygComposerView {
    /// Sets the tintColor of the WYSIWYG textView, if not used the default value is Color.accent.
    func tintColor(_ tintColor: Color) -> Self {
        var newSelf = self
        newSelf.tintColor = tintColor
        return newSelf
    }
}

// MARK: - Logger

private extension Logger {
    static let textView = Logger(subsystem: subsystem, category: "TextView")
}
