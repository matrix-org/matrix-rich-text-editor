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

import Foundation

protocol ComposerModelWrapperProtocol {
    func setContentFromHtml(html: String) -> ComposerUpdate
    func setContentFromMarkdown(markdown: String) -> ComposerUpdate
    func getContentAsHtml() -> String
    func getContentAsMarkdown() -> String
    func clear() -> ComposerUpdate
    func select(startUtf16Codeunit: UInt32, endUtf16Codeunit: UInt32) -> ComposerUpdate
    func replaceText(newText: String) -> ComposerUpdate
    func replaceTextIn(newText: String, start: UInt32, end: UInt32) -> ComposerUpdate
    func backspace() -> ComposerUpdate
    func enter() -> ComposerUpdate
    func apply(_ action: WysiwygAction) -> ComposerUpdate
    func setLink(link: String) -> ComposerUpdate
    func setLinkWithText(link: String, text: String) -> ComposerUpdate
    func removeLinks() -> ComposerUpdate
    func toTree() -> String
    func getCurrentDomState() -> ComposerState
    func actionStates() -> [ComposerAction: ActionState]
    func getLinkAction() -> LinkAction
}

/// Provides a wrapper around `ComposerModel` that handles failures and reset to
/// a fallback content if needed.
final class ComposerModelWrapper: ComposerModelWrapperProtocol {
    // MARK: - Private

    private var model = newComposerModel()
    private var fallbackContent: (() -> String)?

    // MARK: - Internal

    init(fallbackContent: (() -> String)? = nil) {
        self.fallbackContent = fallbackContent
    }

    func setContentFromHtml(html: String) -> ComposerUpdate {
        execute { try $0.setContentFromHtml(html: html) }
    }

    func setContentFromMarkdown(markdown: String) -> ComposerUpdate {
        execute { try $0.setContentFromMarkdown(markdown: markdown) }
    }

    func getContentAsHtml() -> String {
        model.getContentAsHtml()
    }

    func getContentAsMarkdown() -> String {
        model.getContentAsMarkdown()
    }

    func clear() -> ComposerUpdate {
        execute { try $0.clear() }
    }

    func select(startUtf16Codeunit: UInt32, endUtf16Codeunit: UInt32) -> ComposerUpdate {
        execute { try $0.select(startUtf16Codeunit: startUtf16Codeunit, endUtf16Codeunit: endUtf16Codeunit) }
    }

    func replaceText(newText: String) -> ComposerUpdate {
        execute { try $0.replaceText(newText: newText) }
    }

    func replaceTextIn(newText: String, start: UInt32, end: UInt32) -> ComposerUpdate {
        execute { try $0.replaceTextIn(newText: newText, start: start, end: end) }
    }

    func backspace() -> ComposerUpdate {
        execute { try $0.backspace() }
    }

    func enter() -> ComposerUpdate {
        execute { try $0.enter() }
    }

    func apply(_ action: WysiwygAction) -> ComposerUpdate {
        execute { try $0.apply(action) }
    }

    func setLink(link: String) -> ComposerUpdate {
        execute { try $0.setLink(link: link) }
    }

    func setLinkWithText(link: String, text: String) -> ComposerUpdate {
        execute { try $0.setLinkWithText(link: link, text: text) }
    }

    func removeLinks() -> ComposerUpdate {
        execute { try $0.removeLinks() }
    }

    func toTree() -> String {
        model.toTree()
    }

    func getCurrentDomState() -> ComposerState {
        model.getCurrentDomState()
    }

    func actionStates() -> [ComposerAction: ActionState] {
        model.actionStates()
    }

    func getLinkAction() -> LinkAction {
        model.getLinkAction()
    }
}

// MARK: - Private

private extension ComposerModelWrapper {
    /// Execute some failable action on the model and restore provided fallback content if needed.
    func execute(_ action: @escaping (ComposerModel) throws -> ComposerUpdate) -> ComposerUpdate {
        do {
            let update = try action(model)
            return update
        } catch {
            model = newComposerModel()
            if let fallbackContent {
                do {
                    let update = try model.replaceText(newText: fallbackContent())
                    return update
                } catch {
                    // If setting the fallback content fails, just reset to empty.
                    model = newComposerModel()
                    // Provide an empty update
                    // swiftlint:disable:next force_try
                    return try! model.clear()
                }
            } else {
                // Provide an empty update
                // swiftlint:disable:next force_try
                return try! model.clear()
            }
        }
    }
}
