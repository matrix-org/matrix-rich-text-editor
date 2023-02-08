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

extension WysiwygAction: CaseIterable, Identifiable {
    public static var allCases: [WysiwygAction] = [
        .bold, .italic, .strikeThrough, .underline, .inlineCode,
        .link, .undo, .redo, .orderedList, .unorderedList, .indent, .unindent, .codeBlock, .quote,
    ]

    public var id: String {
        accessibilityIdentifier.rawValue
    }

    /// Compute color for action button.
    ///
    /// - Parameter viewModel: Composer's view model.
    /// - Returns: Tint color that the button should use.
    public func color(_ viewModel: WysiwygComposerViewModel) -> Color {
        switch viewModel.actionStates[composerAction] {
        case .enabled:
            return Color.primary
        case .reversed:
            return Color.accentColor
        default:
            return Color.primary.opacity(0.3)
        }
    }

    /// Compute disabled status for action.
    ///
    /// - Parameter viewModel: Composer's view model.
    /// - Returns: True if the action is disabled, false otherwise.
    public func isDisabled(_ viewModel: WysiwygComposerViewModel) -> Bool {
        viewModel.actionStates[composerAction] == ActionState.disabled
    }

    /// Compute visibility status for action.
    ///
    /// - Parameter viewModel: Composer's view model.
    /// - Returns: True if the action is visible, false otherwise.
    public func isVisible(_ viewModel: WysiwygComposerViewModel) -> Bool {
        switch self {
        case .indent, .unindent:
            return viewModel.isInList
        default:
            return true
        }
    }

    var accessibilityIdentifier: WysiwygSharedAccessibilityIdentifier {
        switch self {
        case .bold:
            return .boldButton
        case .italic:
            return .italicButton
        case .strikeThrough:
            return .strikeThroughButton
        case .underline:
            return .underlineButton
        case .inlineCode:
            return .inlineCodeButton
        case .link:
            return .linkButton
        case .undo:
            return .undoButton
        case .redo:
            return .redoButton
        case .orderedList:
            return .orderedListButton
        case .unorderedList:
            return .unorderedListButton
        case .indent:
            return .indentButton
        case .unindent:
            return .unindentButton
        case .codeBlock:
            return .codeBlockButton
        case .quote:
            return .quoteButton
        }
    }

    /// Returns the name of the system icon that should be used for button display.
    var iconName: String {
        switch self {
        case .bold:
            return "bold"
        case .italic:
            return "italic"
        case .strikeThrough:
            return "strikethrough"
        case .underline:
            return "underline"
        case .inlineCode:
            return "chevron.left.forwardslash.chevron.right"
        case .link:
            return "link"
        case .undo:
            return "arrow.uturn.backward"
        case .redo:
            return "arrow.uturn.forward"
        case .orderedList:
            return "list.number"
        case .unorderedList:
            return "list.bullet"
        case .indent:
            return "increase.indent"
        case .unindent:
            return "decrease.indent"
        case .codeBlock:
            return "note.text"
        case .quote:
            return "text.quote"
        }
    }
}

private extension WysiwygAction {
    private var composerAction: ComposerAction {
        switch self {
        case .bold:
            return .bold
        case .italic:
            return .italic
        case .strikeThrough:
            return .strikeThrough
        case .underline:
            return .underline
        case .inlineCode:
            return .inlineCode
        case .link:
            return .link
        case .undo:
            return .undo
        case .redo:
            return .redo
        case .orderedList:
            return .orderedList
        case .unorderedList:
            return .unorderedList
        case .indent:
            return .indent
        case .unindent:
            return .unindent
        case .codeBlock:
            return .codeBlock
        case .quote:
            return .quote
        }
    }
}

private extension WysiwygComposerViewModel {
    /// Returns true if we are currently inside a list.
    var isInList: Bool {
        actionStates[.orderedList] == .reversed || actionStates[.unorderedList] == .reversed
    }
}
