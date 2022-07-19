//
//  WysiwygTextView.swift
//  
//
//  Created by Arnaud Ringenbach on 19/07/2022.
//

import SwiftUI

/// A basic view that displays a text passing through the Rust library.
public struct WysiwygTextView: View {
    // MARK: - Public
    public var body: some View {
        Text(createSampleRustString())
    }

    public init() {}

    // MARK: - Private
    private func createSampleRustString() -> String {
        let model = newComposerModel()
        let update = model.replaceText(newText: "Test string that goes through Rust")
        let rawHTML: String
        switch update.textUpdate() {
        case .replaceAll(replacementHtml: let html,
                         selectionStartCodepoint: _,
                         selectionEndCodepoint: _):
            rawHTML = html
        default:
            rawHTML = "Unable to load from Rust"
        }
        return rawHTML
    }
}
