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
import UIKit
import Combine

/// Main view model for the composer. Forwards actions to the Rust model and publishes resulting states.
public class WysiwygComposerViewModel: ObservableObject {
    // MARK: - Public
    /// Published object for the composer content.
    @Published public var content: WysiwygComposerContent = .init()
    /// Published boolean for the composer empty content state.
    @Published public var isContentEmpty: Bool = true
    /// Published value for the composer required height to fit entirely without scrolling.
    @Published public var idealHeight: CGFloat = .zero

    // MARK: - Private
    private var model: ComposerModel
    private var cancellable: AnyCancellable?

    // MARK: - Public
    public init() {
        self.model = newComposerModel()
        // Publish composer empty state.
        cancellable = $content.sink(receiveValue: { [unowned self] content in
            self.isContentEmpty = content.plainText.isEmpty
        })
    }

    /// Clear the content of the composer.
    public func clearContent() {
        self.model = newComposerModel()
        self.content = WysiwygComposerContent()
    }
}

// MARK: - Internal
extension WysiwygComposerViewModel {
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

    /// Notify that the text view content has changed.
    ///
    /// - Parameter textView: The composer's text view.
    func didUpdateText(textView: UITextView) {
        self.updateIdealHeightIfNeeded(textView)
    }

    /// Apply bold formatting to the current selection.
    func applyBold() {
        let update = self.model.bold()
        self.applyUpdate(update)
    }
}

// MARK: - Private
private extension WysiwygComposerViewModel {
    /// Apply given composer update to the composer.
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
                self.content = WysiwygComposerContent(
                    plainText: attributed.string,
                    html: html,
                    attributed: attributed,
                    selection: textSelection)
                Logger.composer.debug("HTML from Rust: \(html), selection: \(textSelection)")
            } catch {
                Logger.composer.error("Unable to update composer display: \(error.localizedDescription)")
            }
        default:
            break
        }
    }

    /// Update the composer total required height if it has changed.
    ///
    /// - Parameters:
    ///   - textView: The composer's text view.
    func updateIdealHeightIfNeeded(_ textView: UITextView) {
        // TODO: remove magic numbers
        let idealHeight = 50 + 16 + 8 + textView
            .sizeThatFits(CGSize(width: textView.bounds.size.width,
                                 height: CGFloat.greatestFiniteMagnitude)
            )
            .height
        self.idealHeight = idealHeight
    }
}
