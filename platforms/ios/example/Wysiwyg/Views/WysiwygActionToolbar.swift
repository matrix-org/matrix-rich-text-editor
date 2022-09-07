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
    var toolbarAction: (WysiwygAction) -> ()

    var body: some View {
        HStack {
            ForEach(WysiwygAction.allCases) { action in
                Button {
                    toolbarAction(action)
                } label: {
                    Image(systemName: action.iconName)
                        .renderingMode(.template)
                        .foregroundColor(action.color(viewModel))
                }
                .buttonStyle(.automatic)
                .accessibilityIdentifier(action.accessibilityIdentifier)
            }

        }
        .frame(width: nil, height: 50, alignment: .center)
    }
}
