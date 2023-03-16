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
    @State private var linkAction: LinkAction?
    
    var body: some View {
        HStack {
            ForEach(WysiwygAction.allCases.filter { $0.isVisible(viewModel) }) { action in
                Button {
                    if action == .link {
                        linkAttributedRange = viewModel.attributedContent.selection
                        linkAction = viewModel.getLinkAction()
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
        .alert(isPresented: $isShowingUrlAlert, makeAlertConfig())
        .frame(width: nil, height: 50, alignment: .center)
    }
    
    func makeAlertConfig() -> AlertConfig {
        var actions: [AlertConfig.Action] = [.cancel(title: "Cancel")]
        let createLinkTitle = "Create Link"
        switch linkAction {
        case .create:
            let singleTextAction: ([String]) -> Void = { strings in
                let urlString = strings[0]
                viewModel.select(range: linkAttributedRange)
                viewModel.applyLinkOperation(.setLink(urlString: urlString))
            }
            actions.append(createAction(singleTextAction: singleTextAction))
            return AlertConfig(title: createLinkTitle, actions: actions)
        case .createWithText:
            let doubleTextAction: ([String]) -> Void = { strings in
                let urlString = strings[0]
                let text = strings[1]
                viewModel.select(range: linkAttributedRange)
                viewModel.applyLinkOperation(.createLink(urlString: urlString, text: text))
            }
            actions.append(createWithTextAction(doubleTextAction: doubleTextAction))
            return AlertConfig(title: createLinkTitle, actions: actions)
        case let .edit(url, text):
            let editLinktitle = "Edit Link"
            let doubleTextAction: ([String]) -> Void = { strings in
                let urlString = strings[0]
                let text = strings[1]
                viewModel.select(range: linkAttributedRange)
                viewModel.applyLinkOperation(.editLink(urlString: urlString, text: text))
            }
            actions.append(editTextAction(doubleTextAction: doubleTextAction, url: url, text: text))
            let removeAction = {
                viewModel.select(range: linkAttributedRange)
                viewModel.applyLinkOperation(.removeLinks)
            }
            actions.append(.destructive(title: "Remove", action: removeAction))
            return AlertConfig(title: editLinktitle, actions: actions)
        case .none:
            return AlertConfig(title: "", actions: actions)
        }
    }
}

private extension WysiwygActionToolbar {
    private func createAction(singleTextAction: @escaping ([String]) -> Void) -> AlertConfig.Action {
        .textAction(
            title: "Ok",
            textFieldsData: [
                .init(
                    accessibilityIdentifier: .linkUrlTextField,
                    placeholder: "URL",
                    defaultValue: nil
                ),
            ],
            action: singleTextAction
        )
    }

    private func createWithTextAction(doubleTextAction: @escaping ([String]) -> Void) -> AlertConfig.Action {
        .textAction(
            title: "Ok",
            textFieldsData: [
                .init(
                    accessibilityIdentifier: .linkUrlTextField,
                    placeholder: "URL",
                    defaultValue: nil
                ),
                .init(
                    accessibilityIdentifier: .linkTextTextField,
                    placeholder: "Text",
                    defaultValue: nil
                ),
            ],
            action: doubleTextAction
        )
    }

    private func editTextAction(doubleTextAction: @escaping ([String]) -> Void, url: String, text: String) -> AlertConfig.Action {
        .textAction(
            title: "Ok",
            textFieldsData: [
                .init(
                    accessibilityIdentifier: .linkUrlTextField,
                    placeholder: "URL",
                    defaultValue: url
                ),
                .init(
                    accessibilityIdentifier: .linkTextTextField,
                    placeholder: "Text",
                    defaultValue: text
                ),
            ],
            action: doubleTextAction
        )
    }
}
