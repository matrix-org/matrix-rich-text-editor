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
import WysiwygComposer

/// Example SwiftUI view that adds a WysiwygComposer as well as a button that
/// displays the message (+ HTML representation) the composer would send.
struct ContentView: View {
    /// A composer content "saved" and displayed upon hitting the send button.
    @State private var sentMessage: WysiwygComposerContent?
    @State private var tree: String?
    /// View model for the composer.
    @StateObject private var viewModel = WysiwygComposerViewModel(
        minHeight: WysiwygSharedConstants.composerMinHeight,
        maxExpandedHeight: WysiwygSharedConstants.composerMaxExtendedHeight
    )

    var body: some View {
        Spacer()
            .frame(width: nil, height: 50, alignment: .center)
        Composer(viewModel: viewModel)
        Button("Min/Max") {
            viewModel.maximised.toggle()
        }
        .accessibilityIdentifier(.minMaxButton)
        Button("Plain/Rich") {
            viewModel.plainTextMode.toggle()
        }
        .accessibilityIdentifier(.plainRichButton)
        Button("Send") {
            sentMessage = viewModel.content
            viewModel.clearContent()
        }
        .disabled(viewModel.isContentEmpty)
        .accessibilityIdentifier(.sendButton)
        Button("Show tree") {
            tree = viewModel.treeRepresentation()
        }
        ScrollView {
            if let tree = tree {
                Text(tree)
                    .font(.system(size: 11.0, weight: .regular, design: .monospaced))
                    .multilineTextAlignment(.leading)
            }
            if let sentMessage = sentMessage {
                VStack {
                    HStack {
                        Text("Content:")
                        Text(sentMessage.markdown)
                            .accessibilityIdentifier(.contentText)
                    }
                    HStack {
                        Text("HTML:")
                        Text(sentMessage.html)
                            .accessibilityIdentifier(.htmlContentText)
                    }
                }
            }
        }
        Spacer()
    }
}
