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
    public private(set) var textView = PlaceholdableTextView()
    /// The composer minimal height.
    public let minHeight: CGFloat
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
    
    /// The color that will be used to display links
    public var linkColor: UIColor {
        didSet {
            // In case of a color change, this will refresh the attributed text
            textView.linkTextAttributes[.foregroundColor] = linkColor
            let update = model.setContentFromHtml(html: content.html)
            applyUpdate(update)
            updateTextView()
        }
    }
    
    /// The color that will be used for the background of code blocks
    public var codeBackgroundColor: UIColor {
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
            updateCompressedHeightIfNeeded()
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
        if plainTextMode {
            _ = model.setContentFromMarkdown(markdown: textView.text)
        }
        return WysiwygComposerContent(markdown: model.getContentAsMarkdown(),
                                      html: model.getContentAsHtml())
    }

    // MARK: - Private

    private var model: ComposerModel
    private var cancellables = Set<AnyCancellable>()
    private var defaultTextAttributes: [NSAttributedString.Key: Any] {
        [.font: UIFont.preferredFont(forTextStyle: .body),
         .foregroundColor: textColor]
    }

    private var hasPendingFormats = false

    // MARK: - Public

    public init(minHeight: CGFloat = 22,
                maxCompressedHeight: CGFloat = 200,
                maxExpandedHeight: CGFloat = 300,
                textColor: UIColor = .label,
                linkColor: UIColor = .link,
                codeBackgroundColor: UIColor = .systemGray5) {
        self.minHeight = minHeight
        self.maxCompressedHeight = maxCompressedHeight
        self.maxExpandedHeight = maxExpandedHeight
        self.textColor = textColor
        self.linkColor = linkColor
        self.codeBackgroundColor = codeBackgroundColor
        textView.linkTextAttributes[.foregroundColor] = linkColor
        model = newComposerModel()
        // Publish composer empty state.
        $attributedContent.sink { [unowned self] content in
            self.isContentEmpty = content.text.length == 0
        }
        .store(in: &cancellables)
        
        $isContentEmpty
            .removeDuplicates()
            .sink { [unowned self] isContentEmpty in
                self.textView.shouldShowPlaceholder = isContentEmpty
            }
            .store(in: &cancellables)
        
        $idealHeight
            .removeDuplicates()
            .sink { [weak self] _ in
                guard let self = self else { return }
                // Improves a lot the user experience by keeping the selected range always visible when there are changes in the size.
                DispatchQueue.main.async {
                    self.textView.scrollRangeToVisible(self.textView.selectedRange)
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
    }

    /// Apply given action to the composer.
    ///
    /// - Parameters:
    ///   - action: Action to apply.
    func apply(_ action: WysiwygAction) {
        Logger.viewModel.logDebug([attributedContent.logSelection,
                                   "Apply action: \(action)"],
                                  functionName: #function)
        guard let update = model.apply(action) else { return }
        if update.textUpdate() == .keep {
            hasPendingFormats = true
        }
        applyUpdate(update)
    }

    /// Sets given HTML as the current content of the composer.
    ///
    /// - Parameters:
    ///   - html: HTML content to apply
    func setHtmlContent(_ html: String) {
        let update = model.setContentFromHtml(html: html)
        applyUpdate(update)
    }

    /// Clear the content of the composer.
    func clearContent() {
        if plainTextMode {
            textView.attributedText = NSAttributedString(string: "", attributes: defaultTextAttributes)
        } else {
            applyUpdate(model.clear())
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
        
        // This fixes a bug where the attributed string keeps link and inline code formatting
        // when they are the last formatting to be deleted
        if textView.attributedText.length == 0 {
            textView.typingAttributes = defaultTextAttributes
        }

        let update: ComposerUpdate
        let shouldAcceptChange: Bool

        if range != attributedContent.selection {
            select(range: range)
        }

        if attributedContent.selection.length == 0, replacementText == "" {
            update = model.backspace()
            shouldAcceptChange = false
        } else if replacementText.count == 1, replacementText[String.Index(utf16Offset: 0, in: replacementText)].isNewline {
            update = model.enter()
            shouldAcceptChange = false
        } else {
            update = model.replaceText(newText: replacementText)
            shouldAcceptChange = true
        }
        
        // Reconciliates the model with the text any time the link state changes
        // this adjusts an iOS behaviour that extends a link when typing after it
        // which does not reflect the model state.
        switch update.menuState() {
        case let .update(newState):
            if newState[.link] != actionStates[.link] {
                applyUpdate(update, skipTextViewUpdate: true)
                textView.apply(attributedContent)
                updateCompressedHeightIfNeeded()
                return false
            }
        default: break
        }
        
        applyUpdate(update, skipTextViewUpdate: shouldAcceptChange)
        return shouldAcceptChange
    }

    func select(range: NSRange) {
        do {
            guard let text = textView.attributedText else { return }
            let htmlSelection = try text.htmlRange(from: range)
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

    func didUpdateText() {
        if plainTextMode {
            if textView.text.isEmpty != isContentEmpty {
                isContentEmpty = textView.text.isEmpty
            }
        } else {
            reconciliateIfNeeded()
            applyPendingFormatsIfNeeded()
        }

        textView.shouldShowPlaceholder = textView.attributedText.length == 0
        updateCompressedHeightIfNeeded()
    }
    
    func applyLinkOperation(_ linkOperation: WysiwygLinkOperation) {
        let update: ComposerUpdate
        switch linkOperation {
        case let .createLink(urlString, text):
            update = model.setLinkWithText(link: urlString, text: text)
        case let .setLink(urlString):
            update = model.setLink(link: urlString)
        case .removeLinks:
            update = model.removeLinks()
        }
        applyUpdate(update)
    }
    
    func getLinkAction() -> LinkAction {
        model.getLinkAction()
    }
}

// MARK: - Private

private extension WysiwygComposerViewModel {
    func updateTextView() {
        didUpdateText()
    }

    /// Apply given composer update to the composer.
    ///
    /// - Parameters:
    ///   - update: ComposerUpdate to apply.
    ///   - skipTextViewUpdate: A boolean indicating whether updating the text view should be skipped.
    func applyUpdate(_ update: ComposerUpdateProtocol, skipTextViewUpdate: Bool = false) {
        switch update.textUpdate() {
        case let .replaceAll(replacementHtml: codeUnits,
                             startUtf16Codeunit: start,
                             endUtf16Codeunit: end):
            applyReplaceAll(codeUnits: codeUnits, start: start, end: end)
            // Note: this makes replaceAll act like .keep on cases where we expect the text
            // view to be properly updated by the system.
            if !skipTextViewUpdate {
                textView.apply(attributedContent)
                updateCompressedHeightIfNeeded()
            }
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
            let attributed = try HTMLParser.parse(
                html: html,
                textColor: textColor,
                linkColor: linkColor,
                codeBackgroundColor: codeBackgroundColor
            )
            // FIXME: handle error for out of bounds index
            let htmlSelection = NSRange(location: Int(start), length: Int(end - start))
            let textSelection = try attributed.attributedRange(from: htmlSelection)
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
            let textSelection = try attributedContent.text.attributedRange(from: htmlSelection)
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
            let attributed = NSAttributedString(string: model.getContentAsMarkdown(),
                                                attributes: defaultTextAttributes)
            textView.attributedText = attributed
        } else {
            let update = model.setContentFromMarkdown(markdown: textView.text)
            applyUpdate(update)
            updateTextView()
        }
    }

    /// Reconciliate the content of the model with the content of the text view.
    func reconciliateIfNeeded() {
        do {
            guard let replacement = try StringDiffer.replacement(from: attributedContent.text.string,
                                                                 to: textView.text ?? "") else {
                return
            }
            // Reconciliate
            let rustRange = try attributedContent.text.htmlRange(from: replacement.range)

            let replaceUpdate = model.replaceTextIn(newText: replacement.text,
                                                    start: UInt32(rustRange.location),
                                                    end: UInt32(rustRange.upperBound))
            applyUpdate(replaceUpdate, skipTextViewUpdate: true)

            // Resync selectedRange
            let rustSelection = try textView.attributedText.htmlRange(from: textView.selectedRange)
            let selectUpdate = model.select(startUtf16Codeunit: UInt32(rustSelection.location),
                                            endUtf16Codeunit: UInt32(rustSelection.upperBound))
            applyUpdate(selectUpdate)

            Logger.viewModel.logDebug(["Reconciliate from \"\(attributedContent.text.string)\" to \"\(textView.text ?? "")\""],
                                      functionName: #function)
        } catch {
            switch error {
            case StringDifferError.tooComplicated,
                 StringDifferError.insertionsDontMatchRemovals:
                // Restore from the model, as otherwise the composer will enter a broken state
                textView.apply(attributedContent)
                updateCompressedHeightIfNeeded()
                Logger.viewModel.logError(["Reconciliate failed, content has been restored from the model"],
                                          functionName: #function)
            case AttributedRangeError.outOfBoundsAttributedIndex,
                 AttributedRangeError.outOfBoundsHtmlIndex:
                // Just log here for now, the composer is already in a broken state
                Logger.viewModel.logError(["Reconciliate failed due to out of bounds indexes"],
                                          functionName: #function)
            default:
                break
            }
        }
    }

    /// Updates the text view with the current content if we have some pending formats
    /// to apply (e.g. we hit the bold button with no selection).
    func applyPendingFormatsIfNeeded() {
        guard hasPendingFormats else { return }

        textView.apply(attributedContent)
        updateCompressedHeightIfNeeded()
        hasPendingFormats = false
    }
}

// MARK: - Logger

private extension Logger {
    static let viewModel = Logger(subsystem: subsystem, category: "ViewModel")
}
