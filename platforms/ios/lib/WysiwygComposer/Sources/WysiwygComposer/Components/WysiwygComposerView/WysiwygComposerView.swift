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

import SwiftUI
import OSLog

/// Provides a SwiftUI displayable view for the composer UITextView component.
struct WysiwygComposerView: UIViewRepresentable {
    // MARK: - Internal
    var viewState: WysiwygComposerViewState
    var replaceText: (NSAttributedString, NSRange, String) -> ()
    var select: (NSAttributedString, NSRange) -> ()
    var didUpdateText: (UITextView) -> ()

    func makeUIView(context: Context) -> UITextView {
        let textView = UITextView()

        textView.accessibilityIdentifier = "WysiwygComposer"
        textView.font = UIFont(name: "Times New Roman", size: 12.0)
        textView.autocapitalizationType = .sentences
        textView.isSelectable = true
        textView.isUserInteractionEnabled = true
        textView.delegate = context.coordinator
        textView.textStorage.delegate = context.coordinator
        return textView
    }

    func updateUIView(_ uiView: UITextView, context: Context) {
        Logger.composer.debug("New text: \(viewState.displayText.string)")
        uiView.attributedText = viewState.displayText
        uiView.selectedRange = viewState.textSelection
        context.coordinator.didUpdateText(uiView)
    }

    func makeCoordinator() -> Coordinator {
        Coordinator(replaceText, select, didUpdateText)
    }

    /// Coordinates UIKit communication.
    class Coordinator: NSObject, UITextViewDelegate, NSTextStorageDelegate {
        var replaceText: (NSAttributedString, NSRange, String) -> ()
        var select: (NSAttributedString, NSRange) -> ()
        var didUpdateText: (UITextView) -> ()

        init(_ replaceText: @escaping (NSAttributedString, NSRange, String) -> (),
             _ select: @escaping (NSAttributedString, NSRange) -> (),
             _ didUpdateText: @escaping (UITextView) -> ()) {
            self.replaceText = replaceText
            self.select = select
            self.didUpdateText = didUpdateText
        }

        func textView(_ textView: UITextView, shouldChangeTextIn range: NSRange, replacementText text: String) -> Bool {
            self.replaceText(textView.attributedText, range, text)
            return false
        }

        // never called yet
        func textViewDidChange(_ textView: UITextView) {
            self.didUpdateText(textView)
        }

        func textViewDidChangeSelection(_ textView: UITextView) {
            self.select(textView.attributedText, textView.selectedRange)
        }
    }
}
