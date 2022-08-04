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
    /// Displayed content and HTML representation.
    @State private var content: MessageContent = .init()
    /// Height required by the composer.
    @State private var requiredHeight: CGFloat = 104.0
    /// Whether the composer content is currently empty.
    @State private var isTextViewEmpty: Bool = true
    /// A composer content "saved" and displayed upon hitting the send button.
    @State private var sentMessage: MessageContent?

    var body: some View {
        Spacer()
            .frame(width: nil, height: 50, alignment: .center)
        WysiwygView()
            // Register preference changes to retrieve the statuses of the composer.
            .onPreferenceChange(MessageContentPreferenceKey.self) { content in
                self.content = content
            }
            .onPreferenceChange(RequiredHeightPreferenceKey.self) { height in
                self.requiredHeight = height
            }
            .onPreferenceChange(IsEmptyContentPreferenceKey.self) { isEmpty in
                self.isTextViewEmpty = isEmpty
            }
            .frame(maxHeight: min(requiredHeight, 250),
                   alignment: .center)
        Button("Send") {
            sentMessage = content
        }
        .disabled(isTextViewEmpty)
        .accessibilityIdentifier("WysiwygSendButton")
        if let sentMessage = sentMessage {
            Text("Content: \(sentMessage.plainText)\nHTML: \(sentMessage.html)")
                .accessibilityIdentifier("WysiwygContentText")
        }
        Spacer()
    }
}
