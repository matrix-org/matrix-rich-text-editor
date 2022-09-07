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

import WysiwygComposer
import SwiftUI

extension WysiwygAction: CaseIterable, Identifiable {
    public static var allCases: [WysiwygAction] = [
        .bold, .italic, .strikethrough, .underline,
        .undo, .redo, .orderedList, .unorderedList
    ]

    public var id: String {
        accessibilityIdentifier.rawValue
    }

    public func color(_ viewModel: WysiwygComposerViewModel) -> Color {
        switch self {
        case .orderedList:
            return viewModel.activeButtons.contains(.orderedList) ? .blue : .black
        case .unorderedList:
            return viewModel.activeButtons.contains(.unorderedList) ? .blue : .black
        default:
            return .black
        }
    }

    var accessibilityIdentifier: WysiwygSharedAccessibilityIdentifier {
        switch self {
        case .bold:
            return .boldButton
        case .italic:
            return .italicButton
        case .strikethrough:
            return .strikeThroughButton
        case .underline:
            return .underlineButton
        case .undo:
            return .undoButton
        case .redo:
            return .redoButton
        case .orderedList:
            return .orderedListButton
        case .unorderedList:
            return .unorderedListButton
        }
    }

    var iconName: String {
        switch self {
        case .bold:
            return "bold"
        case .italic:
            return "italic"
        case .strikethrough:
            return "strikethrough"
        case .underline:
            return "underline"
        case .undo:
            return "arrow.uturn.backward"
        case .redo:
            return "arrow.uturn.forward"
        case .orderedList:
            return "list.number"
        case .unorderedList:
            return "list.bullet"
        }
    }
}
