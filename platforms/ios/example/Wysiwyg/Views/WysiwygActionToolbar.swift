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

struct WysiwygActionToolbar: View {
    @EnvironmentObject private var viewModel: WysiwygComposerViewModel
    var toolbarAction: (WysiwygAction) -> Void
    @State private var isShowingUrlAlert = false
    @State private var linkAttributedRange = NSRange.zero

    var body: some View {
        HStack {
            ForEach(WysiwygAction.allCases) { action in
                Button {
                    if action == .link(url: "unset") {
                        linkAttributedRange = viewModel.content.attributedSelection
                        isShowingUrlAlert = true
                    } else {
                        toolbarAction(action)
                    }
                } label: {
                    Image(systemName: action.iconName)
                        .renderingMode(.template)
                        .foregroundColor(action.color(viewModel))
                }
                .disabled(action.isDisabled(viewModel))
                .buttonStyle(.automatic)
                .accessibilityIdentifier(action.accessibilityIdentifier)
            }
        }
        .alert(isPresented: $isShowingUrlAlert, AlertConfig(title: "Enter URL", action: { url in
            guard let url = url else { return }
            // Note: the selection needs to be restored because of an issue with SwiftUI
            // integrating multiple UITextField/UITextView breaking selection.
            viewModel.select(text: viewModel.content.attributed, range: linkAttributedRange)
            let action: WysiwygAction = .link(url: url)
            toolbarAction(action)
        }))
        .frame(width: nil, height: 50, alignment: .center)
    }
}
