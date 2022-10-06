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

import Combine
import Foundation
import OSLog
import UIKit

/// Main view model for the composer. Forwards actions to the Rust model and publishes resulting states.
public class WysiwygComposerViewModel: ObservableObject {
    // MARK: - Public

    /// Published object for the composer content.
    @Published public var content: WysiwygComposerContent = .init()
    /// Published boolean for the composer empty content state.
    @Published public var isContentEmpty = true
    /// Published value for the composer required height to fit entirely without scrolling.
    @Published public var idealHeight: CGFloat = .zero
    /// Published value for the composer current expected reversed actions
    /// (e.g. calling `bold` will effectively un-bold the current selection).
    @Published public var reversedActions: [ComposerAction] = []
    /// Published value for the composer current expected disabled actions.
    @Published public var disabledActions: [ComposerAction] = []
    /// Published value for the composer maximised state.
    @Published public var maximised = false {
        didSet {
            updateIdealHeight()
        }
    }

    // MARK: - Private

    private var model: ComposerModel
    private var cancellable: AnyCancellable?
    private let minHeight: CGFloat
    private let maxHeight: CGFloat
    private var compressedHeight: CGFloat = .zero {
        didSet {
            updateIdealHeight()
        }
    }

    // MARK: - Public

    public init(minHeight: CGFloat = 20, maxHeight: CGFloat = 200) {
        self.minHeight = minHeight
        self.maxHeight = maxHeight
        model = newComposerModel()
        // Publish composer empty state.
        cancellable = $content.sink(receiveValue: { [unowned self] content in
            self.isContentEmpty = content.plainText.isEmpty
        })
    }

    /// Apply any additional setup required.
    /// Should be called when the view appears.
    public func setup() {
        applyUpdate(model.replaceAllHtml(html: ""))
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
            let update = model.select(startUtf16Codeunit: UInt32(htmlSelection.location),
                                      endUtf16Codeunit: UInt32(htmlSelection.upperBound))

            applyUpdate(update)
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
        Logger.viewModel.logDebug([content.logAttributedSelection,
                                   "Apply action: \(action)"],
                                  functionName: #function)
        let update: ComposerUpdate
        switch action {
        case .bold:
            update = model.bold()
        case .italic:
            update = model.italic()
        case .strikeThrough:
            update = model.strikeThrough()
        case .underline:
            update = model.underline()
        case .inlineCode:
            update = model.inlineCode()
        case let .link(url: url):
            update = model.setLink(newText: url)
        case .undo:
            update = model.undo()
        case .redo:
            update = model.redo()
        case .orderedList:
            update = model.orderedList()
        case .unorderedList:
            update = model.unorderedList()
        }
        applyUpdate(update)
    }

    /// Clear the content of the composer.
    public func clearContent() {
        model = newComposerModel()
        content = WysiwygComposerContent()
    }

    /// Returns a textual representation of the composer model as a tree.
    public func treeRepresentation() -> String {
        model.toTree()
    }
}

// MARK: - Internal

public extension WysiwygComposerViewModel {
    /// Replace text in the model.
    ///
    /// - Parameters:
    ///   - text: Text currently displayed in the composer.
    ///   - range: Range to replace.
    ///   - replacementText: Replacement text to apply.
    func replaceText(_ text: NSAttributedString, range: NSRange, replacementText: String) -> Bool {
        let update: ComposerUpdate
        let shouldAcceptChange: Bool

        if range != content.attributedSelection {
            select(text: text, range: range)
        }

        if content.attributedSelection.length == 0, replacementText == "" {
            Logger.viewModel.logDebug(["Ignored an empty replacement"],
                                      functionName: #function)
            return false
        }

        if replacementText.count == 1, replacementText[String.Index(utf16Offset: 0, in: replacementText)].isNewline {
            update = model.enter()
            shouldAcceptChange = false
        } else {
            update = model.replaceText(newText: replacementText)
            shouldAcceptChange = true
        }

        applyUpdate(update)
        return shouldAcceptChange
    }

    /// Notify that the text view content has changed.
    ///
    /// - Parameter textView: The composer's text view.
    func didUpdateText(textView: UITextView) {
        // Reconciliate
        if textView.attributedText != content.attributed {
            Logger.viewModel.logDebug(["Reconciliate from \"\(textView.text ?? "")\" to \"\(content.plainText)\""],
                                      functionName: #function)
            textView.apply(content)
        }

        updateCompressedHeightIfNeeded(textView)
    }
}

// MARK: - Private

private extension WysiwygComposerViewModel {
    /// Apply given composer update to the composer.
    ///
    /// - Parameter update: ComposerUpdate to apply.
    func applyUpdate(_ update: ComposerUpdate) {
        switch update.textUpdate() {
        case let .replaceAll(replacementHtml: codeUnits,
                             startUtf16Codeunit: start,
                             endUtf16Codeunit: end):
            applyReplaceAll(codeUnits: codeUnits, start: start, end: end)
        case let .select(startUtf16Codeunit: start,
                         endUtf16Codeunit: end):
            applySelect(start: start, end: end)
        case .keep:
            break
        }

        switch update.menuState() {
        case let .update(reversedActions: reversedActions,
                         disabledActions: disabledActions):
            self.reversedActions = reversedActions
            self.disabledActions = disabledActions
        default:
            break
        }
    }

    /// Apply a replaceAll update to the composer
    ///
    /// - Parameters:
    ///   - codeUnits: Array of UTF16 code units representing the current HTML.
    ///   - start: Start location for the selection.
    ///   - end: End location for the selection.
    func applyReplaceAll(codeUnits: [UInt16], start: UInt32, end: UInt32) {
        do {
            let html = String(utf16CodeUnits: codeUnits, count: codeUnits.count)
            let htmlWithStyle = generateHtmlBodyWithStyle(htmlFragment: html)
            let attributed = try NSAttributedString(html: htmlWithStyle)
            // FIXME: handle error for out of bounds index
            let htmlSelection = NSRange(location: Int(start), length: Int(end - start))
            // FIXME: temporary workaround as trailing newline should be ignored but are now replacing ZWSP from Rust model
            let textSelection = try attributed.attributedRange(from: htmlSelection,
                                                               shouldIgnoreTrailingNewline: false)
            content = WysiwygComposerContent(
                plainText: attributed.string,
                html: html,
                attributed: attributed,
                attributedSelection: textSelection
            )
            Logger.viewModel.logDebug(["Sel(att): \(textSelection)",
                                       "Sel: \(htmlSelection)",
                                       "HTML: \"\(html)\"",
                                       "replaceAll"],
                                      functionName: #function)
        } catch {
            Logger.viewModel.logError(["Sel: {\(start), \(end - start)}",
                                       "Error: \(error.localizedDescription)",
                                       "replaceAll"],
                                      functionName: #function)
        }
    }

    /// Apply a select update to the composer
    ///
    /// - Parameters:
    ///   - start: Start location for the selection.
    ///   - end: End location for the selection.
    func applySelect(start: UInt32, end: UInt32) {
        do {
            let htmlSelection = NSRange(location: Int(start), length: Int(end - start))
            // FIXME: temporary workaround as trailing newline should be ignored but are now replacing ZWSP from Rust model
            let textSelection = try content.attributed.attributedRange(from: htmlSelection,
                                                                       shouldIgnoreTrailingNewline: false)
            content.attributedSelection = textSelection
            Logger.viewModel.logDebug(["Sel(att): \(textSelection)",
                                       "Sel: \(htmlSelection)"],
                                      functionName: #function)
        } catch {
            Logger.viewModel.logError(["Sel: {\(start), \(end - start)}",
                                       "Error: \(error.localizedDescription)"],
                                      functionName: #function)
        }
    }

    /// Update the composer compressed required height if it has changed.
    ///
    /// - Parameters:
    ///   - textView: The composer's text view.
    func updateCompressedHeightIfNeeded(_ textView: UITextView) {
        let idealTextHeight = textView
            .sizeThatFits(CGSize(width: textView.bounds.size.width,
                                 height: CGFloat.greatestFiniteMagnitude)
            )
            .height
        
        compressedHeight = min(maxHeight, max(minHeight, idealTextHeight))
    }
    
    /// Update the composer ideal height based on the maximised state.
    ///
    func updateIdealHeight() {
        if maximised {
            idealHeight = maxHeight
        } else {
            // This solves the slowdown caused by the "Publishing changes from within view updates" purple warning
            DispatchQueue.main.async {
                self.idealHeight = self.compressedHeight
            }
        }
    }
    
    func generateHtmlBodyWithStyle(htmlFragment: String) -> String {
        "<html><head><style>body {font-family:-apple-system;font:-apple-system-body;}</style></head><body>\(htmlFragment)</body></html>"
    }
}

// MARK: - Logger

private extension Logger {
    static let viewModel = Logger(subsystem: subsystem, category: "ViewModel")
}
