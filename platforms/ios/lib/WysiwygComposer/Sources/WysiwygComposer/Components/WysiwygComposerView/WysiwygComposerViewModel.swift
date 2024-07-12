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
import HTMLParser
import OSLog
import SwiftUI
import UIKit

// swiftlint:disable file_length
/// Main view model for the composer. Forwards actions to the Rust model and publishes resulting states.
public class WysiwygComposerViewModel: WysiwygComposerViewModelProtocol, ObservableObject {
    // MARK: - Public

    /// The textView that the model currently manages, a default text view is provided, but you should always inject it from the UIWrapper
    public lazy var textView = {
        let textView = WysiwygTextView()
        textView.linkTextAttributes[.foregroundColor] = parserStyle.linkColor
        textView.mentionDisplayHelper = mentionDisplayHelper
        textView.apply(attributedContent, committed: &committedAttributedText)
        return textView
    }() {
        didSet {
            textView.linkTextAttributes[.foregroundColor] = parserStyle.linkColor
            textView.mentionDisplayHelper = mentionDisplayHelper
            textView.apply(attributedContent, committed: &committedAttributedText)
        }
    }
        
    /// The composer minimal height.
    public let minHeight: CGFloat
    /// The mention replacer defined by the hosting application.
    public var mentionReplacer: MentionReplacer?
    /// Published object for the composer attributed content.
    @Published public var attributedContent: WysiwygComposerAttributedContent = .init()
    /// Published value for the content of the text view in plain text mode.
    @Published public var plainTextContent = NSAttributedString()
    /// Published boolean for the composer empty content state.
    @Published public var isContentEmpty = true
    /// Published value for the composer ideal height to fit.
    /// Note: with SwiftUI & iOS > 16.0, the `UIViewRepresentable` will
    /// use `sizeThatFits` making registering to that publisher usually unnecessary.
    @Published public var idealHeight: CGFloat = .zero
    /// Published value for the composer current action states.
    @Published public var actionStates: [ComposerAction: ActionState] = [:]
    /// Published value for current detected suggestion pattern.
    @Published public var suggestionPattern: SuggestionPattern?
    /// Published value for the composer maximised state.
    @Published public var maximised = false {
        didSet {
            updateIdealHeight()
        }
    }

    /// Whether the composer should take any keyboard input.
    /// When set to `false`, `replaceText(range:replacementText:)` returns `false` as well.
    public var shouldReplaceText = true

    /// Published value for the composer plain text mode.
    @Published public var plainTextMode = false {
        didSet {
            updatePlainTextMode(plainTextMode)
        }
    }

    /// Style for the HTML parser.
    public var parserStyle: HTMLParserStyle {
        didSet {
            // In case of a color change, this will refresh the attributed text
            textView.linkTextAttributes[.foregroundColor] = parserStyle.linkColor
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
            _ = model.setContentFromMarkdown(markdown: computeMarkdownContent())
        }
        return WysiwygComposerContent(markdown: model.getContentAsMessageMarkdown(),
                                      html: model.getContentAsMessageHtml())
    }
    
    /// The mention helper that will be used by the underlying textView
    public var mentionDisplayHelper: MentionDisplayHelper? {
        didSet {
            textView.mentionDisplayHelper = mentionDisplayHelper
        }
    }

    // MARK: - Private

    private let model: ComposerModelWrapper
    private var cancellables = Set<AnyCancellable>()
    private var defaultTextAttributes: [NSAttributedString.Key: Any] {
        [.font: UIFont.preferredFont(forTextStyle: .body),
         .foregroundColor: parserStyle.textColor]
    }

    private(set) var hasPendingFormats = false
    
    /// This is used to track the text commited to the editor by the user, as opposed to text
    /// that could be in the editor that is not yet committed (e.g. from inline predictive text or dictation ).
    private lazy var committedAttributedText = NSAttributedString(string: "", attributes: defaultTextAttributes)
    
    private var lastReplaceTextUpdate: ReplaceTextUpdate?
    
    /// Wether the view contains uncommitted text(e.g. a predictive suggestion is shown in grey).
    private var hasUncommitedText: Bool {
        textView.attributedText.htmlChars.withNBSP != committedAttributedText.htmlChars.withNBSP
    }
    
    // MARK: - Public

    public init(minHeight: CGFloat = 22,
                maxCompressedHeight: CGFloat = 200,
                maxExpandedHeight: CGFloat = 300,
                parserStyle: HTMLParserStyle = .standard,
                mentionReplacer: MentionReplacer? = nil) {
        self.minHeight = minHeight
        idealHeight = minHeight
        self.maxCompressedHeight = maxCompressedHeight
        self.maxExpandedHeight = maxExpandedHeight
        self.parserStyle = parserStyle
        self.mentionReplacer = mentionReplacer

        model = ComposerModelWrapper()
        model.delegate = self
        // Publish composer empty state.
        $attributedContent.sink { [unowned self] content in
            isContentEmpty = content.text.length == 0 || content.plainText == "\n" // An empty <p> is left when deleting multi-line content.
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

    deinit {
        mentionReplacer = nil
    }
}

// MARK: - Public

public extension WysiwygComposerViewModel {
    /// Apply any additional setup required.
    /// Should be called when the view appears.
    func setup() {
        clearContent()
    }

    /// Apply given action to the composer.
    ///
    /// - Parameters:
    ///   - action: Action to apply.
    func apply(_ action: ComposerAction) {
        Logger.viewModel.logDebug([attributedContent.logSelection,
                                   "Apply action: \(action)"],
                                  functionName: #function)
        let update = model.apply(action)
        if update.textUpdate() == .keep {
            hasPendingFormats = true
        } else if attributedContent.selection.length == 0, action.requiresReapplyFormattingOnEmptySelection {
            // Set pending format if current action requires it.
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
        if plainTextMode {
            updatePlainTextMode(true)
        }
    }

    /// Sets given Markdown as the current content of the composer.
    ///
    /// - Parameters:
    ///   - markdown: Markdown content to apply
    func setMarkdownContent(_ markdown: String) {
        let update = model.setContentFromMarkdown(markdown: markdown)
        applyUpdate(update)
        if plainTextMode {
            updatePlainTextMode(true)
        }
    }

    /// Clear the content of the composer.
    func clearContent() {
        if plainTextMode {
            textView.attributedText = NSAttributedString(string: "", attributes: defaultTextAttributes)
            updateCompressedHeightIfNeeded()
        } else {
            applyUpdate(model.clear())
        }
    }

    /// Returns a textual representation of the composer model as a tree.
    func treeRepresentation() -> String {
        model.toTree()
    }

    /// Set a mention with given pattern. Usually used
    /// to mention a user (@) or a room/channel (#).
    ///
    /// - Parameters:
    ///   - url: The URL to the user/room.
    ///   - name: The display name of the user/room.
    ///   - mentionType: The type of mention.
    func setMention(url: String, name: String, mentionType: WysiwygMentionType) {
        let update: ComposerUpdate
        if let suggestionPattern, suggestionPattern.key == mentionType.patternKey {
            update = model.insertMentionAtSuggestion(url: url,
                                                     text: name,
                                                     suggestion: suggestionPattern,
                                                     attributes: mentionType.attributes)
        } else {
            update = model.insertMention(url: url,
                                         text: name,
                                         attributes: mentionType.attributes)
        }
        applyUpdate(update)
        hasPendingFormats = true
    }
    
    /// Sets the @room mention at the suggestion position
    func setAtRoomMention() {
        let update: ComposerUpdate
        if let suggestionPattern, suggestionPattern.key == .at {
            update = model.insertAtRoomMentionAtSuggestion(suggestionPattern)
        } else {
            update = model.insertAtRoomMention()
        }
        applyUpdate(update)
        hasPendingFormats = true
    }

    /// Set a command with `Slash` pattern.
    ///
    /// - Parameters:
    ///   - name: The name of the command.
    func setCommand(name: String) {
        guard let suggestionPattern, suggestionPattern.key == .slash else { return }
        let update = model.replaceTextSuggestion(newText: name, suggestion: suggestionPattern)
        applyUpdate(update)
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

    // swiftlint:disable cyclomatic_complexity
    func replaceText(range: NSRange, replacementText: String) -> Bool {
        guard shouldReplaceText else {
            return false
        }

        guard !plainTextMode else {
            return true
        }
        
        let nextTextUpdate = ReplaceTextUpdate(date: Date.now, range: range, text: replacementText)
        // This is to specifically to work around an issue when tapping on an inline predictive text suggestion within the text view.
        // Even though we have the delegate disabled during modifications to the textview we still get some duplicate
        // calls to this method in this case specifically. It's very unlikely we would get a valid subsequent call
        // with the same range and replacement text within such a short period of time, so should be safe.
        if let lastReplaceTextUpdate, lastReplaceTextUpdate.shouldDebounce(with: nextTextUpdate) {
            return true
        }
        
        // This fixes a bug where the attributed string keeps link and inline code formatting
        // when they are the last formatting to be deleted
        if textView.attributedText.length == 0 {
            textView.typingAttributes = defaultTextAttributes
        }

        let update: ComposerUpdate
        let skipTextViewUpdate: Bool

        if range != attributedContent.selection {
            select(range: range)
        }
        
        // The system handles certain auto-compelete use-cases with somewhat unusual replacementText/range
        // combinations, some of those edge cases are handled below.
        
        // Are we replacing some selected text by tapping the suggestion toolbar
        // When this happens a range/replacementText of this combination is sent.
        let isReplacingWordWithSuggestion = replacementText == "" && !hasUncommitedText && range.length == 0
        
        // A no-op rte side is required here
        if isReplacingWordWithSuggestion {
            return true
        }
        
        // Are we backspacing from an inline predictive text suggestion.
        // When this happens a range/replacementText of this combination is sent.
        let isExitingPredictiveText = replacementText == ""
            && hasUncommitedText
            && range == attributedContent.selection && range.length == 0
        
        let isNormalBackspace = attributedContent.selection.length == 0 && replacementText == ""
        
        if isNormalBackspace || isExitingPredictiveText {
            update = model.backspace()
            skipTextViewUpdate = false
        } else if replacementText.count == 1, replacementText[String.Index(utf16Offset: 0, in: replacementText)].isNewline {
            update = createEnterUpdate()
            skipTextViewUpdate = false
        } else {
            update = model.replaceText(newText: replacementText)
            skipTextViewUpdate = true
        }
        
        // Reconciliates the model with the text any time the link state changes
        // this adjusts an iOS behaviour that extends a link when typing after it
        // which does not reflect the model state.
        switch update.menuState() {
        case let .update(newState):
            if newState[.link] != actionStates[.link] {
                applyUpdate(update, skipTextViewUpdate: true)
                applyAtributedContent()
                updateCompressedHeightIfNeeded()
                return false
            }
        default: break
        }
        
        applyUpdate(update, skipTextViewUpdate: skipTextViewUpdate)
        lastReplaceTextUpdate = nextTextUpdate
        return skipTextViewUpdate
    }

    // swiftlint:enable cyclomatic_complexity
    
    func select(range: NSRange) {
        do {
            guard let text = textView.attributedText, !plainTextMode else { return }
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
            plainTextContent = textView.attributedText
        } else {
            let dotInserted = checkForDoubleSpaceToDotConversion()
            if !dotInserted {
                reconciliateIfNeeded()
            }
            applyPendingFormatsIfNeeded()
        }
        
        updateCompressedHeightIfNeeded()
    }
    
    func checkForDoubleSpaceToDotConversion() -> Bool {
        let text = textView.attributedText.htmlChars.withNBSP
        guard text.count > 0 else {
            return false
        }
        let content = attributedContent.text.htmlChars.withNBSP
        let dotStart = textView.selectedRange.location - 1
        let dotEnd = textView.selectedRange.location
        guard dotStart >= 0, dotEnd <= text.count else {
            return false
        }
        let dotStartIndex = text.index(text.startIndex, offsetBy: dotStart)
        let dotEndIndex = text.index(after: dotStartIndex)
        guard
            dotStartIndex < text.endIndex,
            dotEndIndex <= text.endIndex,
            dotStartIndex < content.endIndex,
            dotEndIndex <= content.endIndex
        else {
            return false
        }
        let dotRange = dotStartIndex..<dotEndIndex
        let textPotentialDot = String(text[dotRange])
        let contentPotentialDot = String(content[dotRange])
        if textPotentialDot == ".", contentPotentialDot != "." {
            let replaceUpdate = model.replaceTextIn(newText: ".",
                                                    start: UInt32(dotStart),
                                                    end: UInt32(dotEnd))
            applyUpdate(replaceUpdate, skipTextViewUpdate: true)
            return true
        }
        return false
    }

    func applyLinkOperation(_ linkOperation: WysiwygLinkOperation) {
        let update: ComposerUpdate
        switch linkOperation {
        case let .createLink(urlString, text):
            update = model.setLinkWithText(url: urlString, text: text, attributes: [])
        case let .setLink(urlString):
            update = model.setLink(url: urlString, attributes: [])
        case .removeLinks:
            update = model.removeLinks()
        }
        applyUpdate(update)
    }
    
    func getLinkAction() -> LinkAction {
        model.getLinkAction()
    }
    
    /// Get the current mentions present in the composer
    func getMentionsState() -> MentionsState {
        model.getMentionsState()
    }

    func enter() {
        applyUpdate(createEnterUpdate(), skipTextViewUpdate: false)
    }

    @available(iOS 16.0, *)
    func getIdealSize(_ proposal: ProposedViewSize) -> CGSize {
        guard let width = proposal.width else { return .zero }

        let idealHeight = textView
            .sizeThatFits(CGSize(width: width, height: CGFloat.greatestFiniteMagnitude))
            .height

        return CGSize(width: width,
                      height: maximised ? maxExpandedHeight : min(maxCompressedHeight, max(minHeight, idealHeight)))
    }
    
    func applyAtributedContent() {
        textView.apply(attributedContent, committed: &committedAttributedText)
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
            if skipTextViewUpdate {
                // We skip updating the text view as the system did that for us but that
                // is not reflected in committedAttributedText yet, so update it.
                committedAttributedText = attributedContent.text
            } else {
                applyAtributedContent()
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

        switch update.menuAction() {
        case .keep:
            break
        case .none:
            suggestionPattern = nil
        case let .suggestion(suggestionPattern: pattern):
            suggestionPattern = pattern
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
            let attributed = try HTMLParser.parse(html: html,
                                                  style: parserStyle,
                                                  mentionReplacer: mentionReplacer)
            // FIXME: handle error for out of bounds index
            let htmlSelection = NSRange(location: Int(start), length: Int(end - start))
            let textSelection = try attributed.attributedRange(from: htmlSelection)
            attributedContent = WysiwygComposerAttributedContent(text: attributed,
                                                                 selection: textSelection,
                                                                 plainText: model.getContentAsPlainText())
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
            if textSelection != attributedContent.selection {
                attributedContent.selection = textSelection
                // Ensure we re-apply required pending formats when switching to a zero-length selection.
                // This fixes selecting in and out of a list / quote / etc
                hasPendingFormats = textSelection.length == 0 && !model.reversedActions.isEmpty
            }
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
            var attributed = NSAttributedString(string: model.getContentAsMarkdown(),
                                                attributes: defaultTextAttributes)
            if let mentionReplacer {
                attributed = mentionReplacer.postProcessMarkdown(in: attributed)
            }
            textView.attributedText = attributed
            updateCompressedHeightIfNeeded()
        } else {
            let update = model.setContentFromMarkdown(markdown: computeMarkdownContent())
            applyUpdate(update)
            updateTextView()
            plainTextContent = NSAttributedString()
        }
    }
    
    /// Reconciliate the content of the model with the content of the text view.
    func reconciliateIfNeeded() {
        do {
            guard !textView.isDictationRunning,
                  let replacement = try StringDiffer.replacement(from: attributedContent.text.htmlChars,
                                                                 to: textView.attributedText.htmlChars) else {
                return
            }
            
            // Don't use reconciliate if the replacement is only latin character languages
            // as it shouldn't be needed. It is needed for CJK lanuages like Japanese Kana.
            if replacement.text.containsLatinAndCommonCharactersOnly {
                return
            }
            
            // Reconciliate
            Logger.viewModel.logDebug(["Reconciliate from \"\(attributedContent.text.string)\" to \"\(textView.text ?? "")\""],
                                      functionName: #function)

            let replaceUpdate = model.replaceTextIn(newText: replacement.text,
                                                    start: UInt32(replacement.range.location),
                                                    end: UInt32(replacement.range.upperBound))
            applyUpdate(replaceUpdate, skipTextViewUpdate: true)

            // Resync selectedRange
            let rustSelection = try textView.attributedText.htmlRange(from: textView.selectedRange)
            let selectUpdate = model.select(startUtf16Codeunit: UInt32(rustSelection.location),
                                            endUtf16Codeunit: UInt32(rustSelection.upperBound))
            applyUpdate(selectUpdate)
        } catch {
            switch error {
            case StringDifferError.tooComplicated,
                 StringDifferError.insertionsDontMatchRemovals:
                // Restore from the model, as otherwise the composer will enter a broken state
                applyAtributedContent()
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
        applyAtributedContent()
        updateCompressedHeightIfNeeded()
        hasPendingFormats = false
    }

    /// Compute the current content of the `UITextView`, as markdown.
    ///
    /// - Returns: A markdown string.
    func computeMarkdownContent() -> String {
        let markdownContent: String
        if let mentionReplacer,
           let attributedText = textView.attributedText {
            // `MentionReplacer` should restore altered content to valid markdown.
            markdownContent = mentionReplacer.restoreMarkdown(in: attributedText)
        } else {
            markdownContent = textView.text
        }

        return markdownContent
    }

    func createEnterUpdate() -> ComposerUpdate {
        let update = model.enter()
        // Pending formats need to be reapplied to the
        // NSAttributedString upon next character input if we
        // are in a structure that might add non-formatted
        // representation chars to it (e.g. NBSP/ZWSP, list prefixes)
        if !model
            .reversedActions
            .isDisjoint(with: [.codeBlock, .quote, .orderedList, .unorderedList]) {
            hasPendingFormats = true
        }
        return update
    }
}

// MARK: - ComposerModelWrapperDelegate

extension WysiwygComposerViewModel: ComposerModelWrapperDelegate {
    func fallbackContent() -> String {
        attributedContent.plainText
    }
}

// MARK: - Logger

private extension Logger {
    static let viewModel = Logger(subsystem: subsystem, category: "ViewModel")
}

private struct ReplaceTextUpdate {
    static let debounceThreshold = 0.1
    var date: Date
    var range: NSRange
    var text: String
}

private extension ReplaceTextUpdate {
    func shouldDebounce(with other: ReplaceTextUpdate) -> Bool {
        range == other.range
            && text == other.text
            && fabs(date.timeIntervalSince(other.date)) < Self.debounceThreshold
    }
}
