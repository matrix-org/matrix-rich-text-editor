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

import Foundation
import OSLog

/// Main view model for the `WysiwygComposerView`. Provides user actions
/// to the Rust model and publishes result `WysiwygComposerViewState`.
class WysiwygComposerViewModel: ObservableObject {
    // MARK: - Internal
    @Published var viewState: WysiwygComposerViewState

    // MARK: - Private
    private var model: ComposerModel

    // MARK: - Init
    init() {
        self.model = newComposerModel()
        self.viewState = WysiwygComposerViewState(
            textSelection: .init(location: 0, length: 0),
            displayText: NSAttributedString()
        )
    }

    // MARK: - Internal
    /// Replace text in the model.
    ///
    /// - Parameters:
    ///   - text: Text currently displayed in the composer.
    ///   - range: Range to replace.
    ///   - replacementText: Replacement text to apply.
    func replaceText(_ text: NSAttributedString, range: NSRange, replacementText: String) {
        let update: ComposerUpdate
        if replacementText == "" {
            // When trying to backspace more than one UTF16 code unit, selection is required.
            if range.length > 1 {
                self.model.select(startUtf16Codeunit: UInt32(range.location),
                                  endUtf16Codeunit: UInt32(range.location+range.length))
            }
            update = self.model.backspace()
        } else {
            update = self.model.replaceText(newText: replacementText)
        }
        self.applyUpdate(update)
    }

    /// Select given range of text within the model.
    ///
    /// - Parameters:
    ///   - text: Text currently displayed in the composer.
    ///   - range: Range to select.
    func select(text: NSAttributedString, range: NSRange) {
        Logger.composer.debug("New selection: \(range) totalLength: \(text.length)")
        self.model.select(startUtf16Codeunit: UInt32(range.location),
                          endUtf16Codeunit: UInt32(range.location+range.length))
    }

    /// Apply bold formatting to the current selection.
    func applyBold() {
        let update = self.model.bold()
        self.applyUpdate(update)
    }
}

// MARK: - Private
private extension WysiwygComposerViewModel {
    /// Apply given composer update to the view state.
    ///
    /// - Parameter update: ComposerUpdate to apply.
    func applyUpdate(_ update: ComposerUpdate) {
        switch update.textUpdate() {
        case .replaceAll(replacementHtml: let codeUnits,
                         startUtf16Codeunit: let start,
                         endUtf16Codeunit: let end):
            let html = String(utf16CodeUnits: codeUnits, count: codeUnits.count).replacingOccurrences(of: "\n", with: "<br>")
            do {
                let attributed = try NSAttributedString(html: html)
                let textSelection = NSRange(location: Int(start), length: Int(end-start))
                self.viewState = WysiwygComposerViewState(
                    textSelection: textSelection,
                    displayText: attributed
                )
                Logger.composer.debug("HTML from Rust: \(html), selection: \(textSelection)")
            } catch {
                Logger.composer.error("Unable to update composer display: \(error.localizedDescription)")
            }
        default:
            break
        }
    }
}
