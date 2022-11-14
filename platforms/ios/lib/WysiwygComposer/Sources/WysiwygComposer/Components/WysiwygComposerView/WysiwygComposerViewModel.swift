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
public class WysiwygComposerViewModel: WysiwygComposerViewModelProtocol, ObservableObject {
    // MARK: - Public

    /// The textView with placeholder support that the model manages
    public var textView: PlaceholdableTextView?
    /// Published object for the composer attributed content.
    @Published public var attributedContent: WysiwygComposerAttributedContent = .init()
    /// Published boolean for the composer empty content state.
    @Published public var isContentEmpty = true
    /// Published value for the composer required height to fit entirely without scrolling.
    @Published public var idealHeight: CGFloat = .zero
    /// Published value for the composer current action states
    @Published public var actionStates: [ComposerAction: ActionState] = [:]
    /// Published value for the composer maximised state.
    @Published public var maximised = false {
        didSet {
            updateIdealHeight()
        }
    }

    /// Published value for the composer plain text mode.
    @Published public var plainTextMode = false {
        didSet {
            updatePlainTextMode(plainTextMode)
        }
    }
    
    /// The current textColor of the attributed string
    public var textColor: UIColor {
        didSet {
            // In case of a color change, this will refresh the attributed text
            let update = model.setContentFromHtml(html: content.html)
            applyUpdate(update)
            updateTextView()
        }
    }
    
    /// The current max allowed height for the textView when maximised
    public var maxExpandedHeight: CGFloat {
        didSet {
            updateIdealHeight()
        }
    }
    
    /// The current max allowed height for the textView when minimised
    public var maxCompressedHeight: CGFloat {
        didSet {
            updateIdealHeight()
        }
    }
    
    /// the current height of the textView when minimised
    public private(set) var compressedHeight: CGFloat = .zero {
        didSet {
            updateIdealHeight()
        }
    }

    /// The current composer content.
    public var content: WysiwygComposerContent {
        if plainTextMode, let plainText = textView?.text {
            _ = model.setContentFromMarkdown(markdown: plainText)
        }
        return WysiwygComposerContent(markdown: model.getContentAsMarkdown(),
                                      html: model.getContentAsHtml())
    }

    // MARK: - Private

    private let minHeight: CGFloat
    private var model: ComposerModel
    private var cancellables = Set<AnyCancellable>()
    private var defaultTextAttributes: [NSAttributedString.Key: Any] {
        [.font: UIFont.preferredFont(forTextStyle: .body),
         .foregroundColor: textColor]
    }

    // MARK: - Public

    public init(minHeight: CGFloat = 22,
                maxCompressedHeight: CGFloat = 200,
                maxExpandedHeight: CGFloat = 300,
                textColor: UIColor = .label) {
        self.minHeight = minHeight
        self.maxCompressedHeight = maxCompressedHeight
        self.maxExpandedHeight = maxExpandedHeight
        self.textColor = textColor
        model = newComposerModel()
        // Publish composer empty state.
        $attributedContent.sink { [unowned self] content in
            self.isContentEmpty = content.text.length == 0
        }
        .store(in: &cancellables)
        
        $isContentEmpty
            .removeDuplicates()
            .sink { [unowned self] isContentEmpty in
                self.textView?.shouldShowPlaceholder = isContentEmpty
            }
            .store(in: &cancellables)
        
        $idealHeight
            .removeDuplicates()
            .sink { [unowned self] _ in
                guard let textView = textView else { return }
                // Improves a lot the user experience by keeping the selected range always visible when there are changes in the size.
                DispatchQueue.main.async {
                    textView.scrollRangeToVisible(textView.selectedRange)
                }
            }
            .store(in: &cancellables)
    }
}

// MARK: - Public

public extension WysiwygComposerViewModel {
    /// Apply any additional setup required.
    /// Should be called when the view appears.
    func setup() {
        // FIXME: multiple textViews sharing the model might unwittingly clear the composer because of this.
        applyUpdate(model.setContentFromHtml(html: ""))
        updateTextView()
    }

    /// Apply given action to the composer.
    ///
    /// - Parameters:
    ///   - action: Action to apply.
    func apply(_ action: WysiwygAction) {
        Logger.viewModel.logDebug([attributedContent.logSelection,
                                   "Apply action: \(action)"],
                                  functionName: #function)
        let update = model.apply(action)
        applyUpdate(update)
        updateTextView()
    }

    /// Sets given HTML as the current content of the composer.
    ///
    /// - Parameters:
    ///   - html: HTML content to apply
    func setHtmlContent(_ html: String) {
        let update = model.setContentFromHtml(html: html)
        applyUpdate(update)
        updateTextView()
    }

    /// Clear the content of the composer.
    func clearContent() {
        if plainTextMode {
            textView?.attributedText = NSAttributedString(string: "", attributes: defaultTextAttributes)
        } else {
            applyUpdate(model.clear())
            updateTextView()
        }
    }

    /// Returns a textual representation of the composer model as a tree.
    func treeRepresentation() -> String {
        model.toTree()
    }
}

// MARK: - WysiwygComposerViewModelProtocol

public extension WysiwygComposerViewModel {
    func updateCompressedHeightIfNeeded() {
        guard let textView = textView else { return }
        let idealTextHeight = textView
            .sizeThatFits(CGSize(width: textView.bounds.size.width,
                                 height: CGFloat.greatestFiniteMagnitude)
            )
            .height

        compressedHeight = min(maxCompressedHeight, max(minHeight, idealTextHeight))
    }

    func replaceText(range: NSRange, replacementText: String) -> Bool {
        guard !plainTextMode else {
            return true
        }

        let update: ComposerUpdate
        let shouldAcceptChange: Bool

        if range != attributedContent.selection {
            select(range: range)
        }

        if attributedContent.selection.length == 0, replacementText == "" {
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
        if !shouldAcceptChange {
            didUpdateText()
        }
        return shouldAcceptChange
    }

    func select(range: NSRange) {
        do {
            guard let text = textView?.attributedText else { return }
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

    func didUpdateText(shouldReconciliate: Bool = true) {
        guard let textView = textView else { return }
        if plainTextMode {
            if textView.text.isEmpty != isContentEmpty {
                isContentEmpty = textView.text.isEmpty
            }
        } else if textView.attributedText != attributedContent.text {
            if shouldReconciliate {
                // Reconciliate
                Logger.viewModel.logDebug(["Reconciliate from \"\(textView.text ?? "")\" to \"\(attributedContent.text)\""],
                                          functionName: #function)
                textView.apply(attributedContent)
            } else {
                textView.shouldShowPlaceholder = textView.attributedText.length == 0
            }
        }

        updateCompressedHeightIfNeeded()
    }
}

// MARK: - Private

private extension WysiwygComposerViewModel {
    func updateTextView() {
        didUpdateText()
    }
    
    /// Apply given composer update to the composer.
    ///
    /// - Parameter update: ComposerUpdate to apply.
    func applyUpdate(_ update: ComposerUpdateProtocol) {
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
        case let .update(actionStates: actionStates):
            self.actionStates = actionStates
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
            let attributed = try HTMLParser.parse(html: html, textColor: textColor)
            // FIXME: handle error for out of bounds index
            let htmlSelection = NSRange(location: Int(start), length: Int(end - start))
            // FIXME: temporary workaround as trailing newline should be ignored but are now replacing ZWSP from Rust model
            let textSelection = try attributed.attributedRange(from: htmlSelection,
                                                               shouldIgnoreTrailingNewline: false)
            attributedContent = WysiwygComposerAttributedContent(text: attributed, selection: textSelection)
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
            let textSelection = try attributedContent.text.attributedRange(from: htmlSelection,
                                                                           shouldIgnoreTrailingNewline: false)
            attributedContent.selection = textSelection
            Logger.viewModel.logDebug(["Sel(att): \(textSelection)",
                                       "Sel: \(htmlSelection)"],
                                      functionName: #function)
        } catch {
            Logger.viewModel.logError(["Sel: {\(start), \(end - start)}",
                                       "Error: \(error.localizedDescription)"],
                                      functionName: #function)
        }
    }
    
    /// Update the composer ideal height based on the maximised state.
    ///
    func updateIdealHeight() {
        if maximised {
            idealHeight = maxExpandedHeight
        } else {
            // This solves the slowdown caused by the "Publishing changes from within view updates" purple warning
            DispatchQueue.main.async {
                self.idealHeight = self.compressedHeight
            }
        }
    }

    /// Updates the view model content for given plain text mode setting.
    ///
    /// - Parameter enabled: whether plain text mode is enabled
    func updatePlainTextMode(_ enabled: Bool) {
        if enabled {
            guard let textView = textView else { return }
            let attributed = NSAttributedString(string: model.getContentAsMarkdown(),
                                                attributes: defaultTextAttributes)
            textView.attributedText = attributed
        } else {
            guard let plainText = textView?.text else { return }
            let update = model.setContentFromMarkdown(markdown: plainText)
            applyUpdate(update)
            updateTextView()
        }
    }
}

// MARK: - Logger

private extension Logger {
    static let viewModel = Logger(subsystem: subsystem, category: "ViewModel")
}
