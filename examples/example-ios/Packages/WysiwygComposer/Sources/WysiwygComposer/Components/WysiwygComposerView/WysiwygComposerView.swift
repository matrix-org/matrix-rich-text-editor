//
//  WysiwygComposerView.swift
//  
//
//  Created by Arnaud Ringenbach on 19/07/2022.
//

import SwiftUI

struct WysiwygComposerView: UIViewRepresentable {

    var viewState: WysiwygComposerViewState
    var change: (String, NSRange, String) -> ()
    var textDidUpdate: (String, NSRange) -> ()
    var textDidChangeSelection: (String, NSRange) -> ()

    func makeUIView(context: Context) -> UITextView {
        let textView = UITextView()

        textView.font = UIFont.systemFont(ofSize: 24)
        textView.autocapitalizationType = .sentences
        textView.isSelectable = true
        textView.isUserInteractionEnabled = true
        textView.delegate = context.coordinator
        textView.textStorage.delegate = context.coordinator
        return textView
    }

    func updateUIView(_ uiView: UITextView, context: Context) {
        guard let data = viewState.html.data(using: .utf16, allowLossyConversion: false) else {
            return
        }
        guard let attrString = try? NSAttributedString(data: data, options: [.documentType: NSAttributedString.DocumentType.html], documentAttributes: nil) else {
            return
        }

        uiView.attributedText = attrString
        uiView.selectedRange = viewState.textSelection
        print("WYSIWYG: Update UI: \(attrString.string) length: \(attrString.length)")
    }

    func makeCoordinator() -> Coordinator {
        Coordinator(change, textDidUpdate, textDidChangeSelection)
    }

    class Coordinator: NSObject, UITextViewDelegate, NSTextStorageDelegate {
        var change: (String, NSRange, String) -> ()
        var textDidUpdate: (String, NSRange) -> ()
        var textDidChangeSelection: (String, NSRange) -> ()

        init(_ change: @escaping (String, NSRange, String) -> (),
             _ textDidUpdate: @escaping (String, NSRange) -> (),
             _ textDidChangeSelection: @escaping (String, NSRange) -> ()) {
            self.change = change
            self.textDidUpdate = textDidUpdate
            self.textDidChangeSelection = textDidChangeSelection
        }

        func textView(_ textView: UITextView, shouldChangeTextIn range: NSRange, replacementText text: String) -> Bool {
            self.change(textView.text, range, text)
            return false
        }

        func textViewDidChange(_ textView: UITextView) {
            textDidUpdate(textView.text, textView.selectedRange)
        }

        func textViewDidChangeSelection(_ textView: UITextView) {
            textDidChangeSelection(textView.text, textView.selectedRange)
        }
    }
}
