//
// Copyright 2022 New Vector Ltd
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

/// Example SwiftUI that embeds the WysiwygComposerView in a bordered container
/// that grows to a max height.
struct Composer: View {
    @ObservedObject var viewModel: WysiwygComposerViewModel
    let itemProviderHelper: WysiwygItemProviderHelper?
    let keyCommandHandler: KeyCommandHandler?
    let pasteHandler: PasteHandler?
    let minTextViewHeight: CGFloat = 20
    let borderHeight: CGFloat = 40
    @FocusState var focused: Bool
    var verticalPadding: CGFloat {
        (borderHeight - minTextViewHeight) / 2
    }
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            let rect = RoundedRectangle(cornerRadius: borderHeight / 2)
            HStack {
                WysiwygComposerView(
                    viewModel: viewModel,
                    itemProviderHelper: itemProviderHelper,
                    keyCommandHandler: keyCommandHandler,
                    pasteHandler: pasteHandler
                )
                .tintColor(.green)
                .placeholder("Placeholder", color: .gray)
                .frame(height: viewModel.idealHeight)
                .padding(.horizontal, 12)
                .onAppear {
                    viewModel.setup()
                }
                .focused($focused, equals: true)
            }
            .padding(.vertical, verticalPadding)
            .clipShape(rect)
            .overlay(rect.stroke(Color.gray, lineWidth: 1))
            .padding(.horizontal, 12)
            .onTapGesture {
                focused = true
            }
            if let suggestion = viewModel.suggestionPattern {
                WysiwygSuggestionList(suggestion: suggestion)
                    .environmentObject(viewModel)
            }
            if !viewModel.plainTextMode {
                WysiwygActionToolbar { action in
                    viewModel.apply(action)
                }
                .environmentObject(viewModel)
                .padding(.horizontal, 16)
            }
        }
    }
}

struct Composer_Previews: PreviewProvider {
    static let viewModel = WysiwygComposerViewModel()
    static var previews: some View {
        Composer(viewModel: viewModel,
                 itemProviderHelper: nil,
                 keyCommandHandler: nil,
                 pasteHandler: nil)
    }
}
