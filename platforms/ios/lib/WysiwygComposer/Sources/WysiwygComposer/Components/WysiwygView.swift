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

/// Composer's main view. Displays a text view and a toolbar for advanced edition capabilities.
public struct WysiwygView: View {
    // MARK: - Public
    public var body: some View {
        VStack {
            WysiwygComposerView(content: viewModel.content,
                                replaceText: viewModel.replaceText,
                                select: viewModel.select,
                                didUpdateText: viewModel.didUpdateText)
            .padding(.all, 8)
            .padding([.leading, .trailing], 8)
            .padding([.top, .bottom], 4)
            .onAppear {
                viewModel.setup()
            }
        }
    }

    public init() { }

    // MARK: - Private
    @EnvironmentObject private var viewModel: WysiwygComposerViewModel
}
