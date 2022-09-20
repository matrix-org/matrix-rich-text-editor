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
    /// Published value for the composer current expected reversed actions (e.g. calling `bold` will effectively un-bold the current selection).
    @Published public var reversedActions: [ComposerAction] = []
    /// Published value for the composer current expected disabled actions.
    @Published public var disabledActions: [ComposerAction] = []

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

    /// Apply any additional setup required.
    /// Should be called when the view appears.
    public func setup() {
        self.applyUpdate(self.model.replaceAllHtml(html: ""))
    }

    /// Select given range of text within the model.
    ///
    /// - Parameters:
    ///   - text: Text currently displayed in the composer.
    ///   - range: Range to select.
    public func select(text: NSAttributedString, range: NSRange) {
        do {
            // FIXME: temporary workaround as trailing newline should be ignored but are now replacing ZWSP from Rust model
            let htmlSelection = try text.htmlRange(from: range,
                                                   shouldIgnoreTrailingNewline: false)
            Logger.viewModel.logDebug(["Sel(att): \(range)",
                                       "Sel: \(htmlSelection)",
                                       "Text: \"\(text.string)\""],
                                       functionName: #function)
            let update = self.model.select(startUtf16Codeunit: UInt32(htmlSelection.location),
                              endUtf16Codeunit: UInt32(htmlSelection.upperBound))

            self.applyUpdate(update)
        } catch {
            Logger.viewModel.logError(["Sel(att): \(range)",
                                       "Error: \(error.localizedDescription)"],
                                      functionName: #function)
        }
    }

    /// Apply given action to the composer.
    ///
    /// - Parameters:
    ///   - action: Action to apply.
    public func apply(_ action: WysiwygAction) {
        let update: ComposerUpdate
        switch action {
        case .bold:
            update = self.model.bold()
        case .italic:
            update = self.model.italic()
        case .strikeThrough:
            update = self.model.strikeThrough()
        case .underline:
            update = self.model.underline()
        case .inlineCode:
            update = self.model.inlineCode()
        case .link(url: let url):
            update = self.model.setLink(newText: url)
        case .undo:
            update = self.model.undo()
        case .redo:
            update = self.model.redo()
        case .orderedList:
            update = self.model.orderedList()
        case .unorderedList:
            update = self.model.unorderedList()
        }
        self.applyUpdate(update)
    }

    /// Clear the content of the composer.
    public func clearContent() {
        self.model = newComposerModel()
        self.content = WysiwygComposerContent()
    }

    /// Returns a textual representation of the composer model as a tree.
    public func treeRepresentation() -> String {
        return self.model.toTree()
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
                self.select(text: text, range: range)
            }
            update = self.model.backspace()
        } else if replacementText == "\n" {
            update = self.model.enter()
        } else {
            update = self.model.replaceText(newText: replacementText)
        }
        self.applyUpdate(update)
    }

    /// Notify that the text view content has changed.
    ///
    /// - Parameter textView: The composer's text view.
    func didUpdateText(textView: UITextView) {
        self.updateIdealHeightIfNeeded(textView)
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
            let html = String(utf16CodeUnits: codeUnits,
                              count: codeUnits.count)
            do {
                let attributed = try NSAttributedString(html: html)
                // FIXME: handle error for out of bounds index
                let htmlSelection = NSRange(location: Int(start), length: Int(end-start))
                // FIXME: temporary workaround as trailing newline should be ignored but are now replacing ZWSP from Rust model
                let textSelection = try attributed.attributedRange(from: htmlSelection,
                                                                   shouldIgnoreTrailingNewline: false)
                self.content = WysiwygComposerContent(
                    plainText: attributed.string,
                    html: html,
                    attributed: attributed,
                    attributedSelection: textSelection)
                Logger.viewModel.logDebug(["Sel(att): \(textSelection)",
                                           "Sel: \(htmlSelection)",
                                           "HTML: \"\(html)\"",
                                           "replaceAll"],
                                          functionName: #function)
            } catch {
                Logger.viewModel.logError(["Sel: {\(start), \(end-start)}",
                                           "Error: \(error.localizedDescription)",
                                           "replaceAll"],
                                          functionName: #function)
            }
        case .select(startUtf16Codeunit: let start,
                     endUtf16Codeunit: let end):
            do {
                let htmlSelection = NSRange(location: Int(start), length: Int(end-start))
                // FIXME: temporary workaround as trailing newline should be ignored but are now replacing ZWSP from Rust model
                let textSelection = try self.content.attributed.attributedRange(from: htmlSelection,
                                                                                shouldIgnoreTrailingNewline: false)
                self.content.attributedSelection = textSelection
                Logger.viewModel.logDebug(["Sel(att): \(textSelection)",
                                           "Sel: \(htmlSelection)"],
                                          functionName: #function)
            } catch {
                Logger.viewModel.logError(["Sel: {\(start), \(end-start)}",
                                           "Error: \(error.localizedDescription)"],
                                          functionName: #function)
            }
        case .keep:
            break
        }

        switch update.menuState() {
        case .update(reversedActions: let reversedActions,
                     disabledActions: let disabledActions):
            self.reversedActions = reversedActions
            self.disabledActions = disabledActions
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

// MARK: - Logger
private extension Logger {
    static let viewModel = Logger(subsystem: subsystem, category: "ViewModel")
}
