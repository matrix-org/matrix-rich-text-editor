//
// Copyright 2023 The Matrix.org Foundation C.I.C
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

struct WysiwygSuggestionList: View {
    @EnvironmentObject private var viewModel: WysiwygComposerViewModel
    var suggestion: SuggestionPattern

    var body: some View {
        HStack {
            VStack(alignment: .leading) {
                switch suggestion.key {
                case .at:
                    let users = Users.filtered(with: suggestion.text)
                    if !users.isEmpty {
                        Text(Users.title).underline()
                        ForEach(users) { user in
                            Button {
                                viewModel.setAtMention(link: user.url, text: user.name)
                            } label: {
                                HStack(spacing: 4) {
                                    Image(systemName: user.iconSystemName)
                                    Text(user.name)
                                }
                            }
                        }
                    }
                case .hash:
                    let rooms = Rooms.filtered(with: suggestion.text)
                    if !rooms.isEmpty {
                        Text(Rooms.title).underline()
                        ForEach(rooms) { room in
                            Button {
                                viewModel.setHashMention(link: room.url, text: room.name)
                            } label: {
                                HStack(spacing: 4) {
                                    Image(systemName: room.iconSystemName)
                                    Text(room.name)
                                }
                            }
                        }
                    }
                case .slash:
                    let commands = Commands.filtered(with: suggestion.text)
                    if !commands.isEmpty {
                        Text(Commands.title).underline()
                        ForEach(Commands.allCases.filter { $0.name.contains("/" + suggestion.text.lowercased()) }) { command in
                            Button {
                                viewModel.setCommand(text: command.name)
                            } label: {
                                HStack(spacing: 4) {
                                    Image(systemName: command.iconSystemName)
                                    Text(command.name)
                                }
                            }
                        }
                    }
                }
            }
            .padding(.horizontal, 8)
            Spacer()
        }
        .overlay(Rectangle().stroke(Color.gray, lineWidth: 1))
        .padding(.horizontal, 12)
    }
}
