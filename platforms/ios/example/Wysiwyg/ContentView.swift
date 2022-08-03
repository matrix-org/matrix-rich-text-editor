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

struct ContentView: View {
    @State private var content: MessageContent = .init()
    @State private var requiredHeight: CGFloat = 104.0
    @State private var isTextViewEmpty: Bool = true
    @State private var sentMessage: MessageContent?

    var body: some View {
        Spacer()
            .frame(width: nil, height: 50, alignment: .center)
        WysiwygView()
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
        if let sentMessage = sentMessage {
            Text("Content: \(sentMessage.plainText)\nHTML: \(sentMessage.html)")
        }
        Spacer()
    }
}
