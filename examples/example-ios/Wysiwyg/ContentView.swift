//
//  ContentView.swift
//  Wysiwyg
//
//  Created by Arnaud Ringenbach on 19/07/2022.
//

import SwiftUI
import WysiwygComposerFFI

struct ContentView: View {
    var body: some View {
        Text(getRustString())
            .padding()
    }

    // FIXME: remove this test func
    private func getRustString() -> String {
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

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}
