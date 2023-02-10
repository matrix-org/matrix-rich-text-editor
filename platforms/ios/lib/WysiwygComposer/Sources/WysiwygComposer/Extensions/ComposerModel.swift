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

extension ComposerModel {
    // swiftlint:disable cyclomatic_complexity
    /// Apply given action to the composer model.
    ///
    /// - Parameters:
    ///   - action: Action to apply.
    func apply(_ action: WysiwygAction) throws -> ComposerUpdate {
        let update: ComposerUpdate
        switch action {
        case .bold:
            update = try bold()
        case .italic:
            update = try italic()
        case .strikeThrough:
            update = try strikeThrough()
        case .underline:
            update = try underline()
        case .inlineCode:
            update = try inlineCode()
        case .undo:
            update = try undo()
        case .redo:
            update = try redo()
        case .orderedList:
            update = try orderedList()
        case .unorderedList:
            update = try unorderedList()
        case .indent:
            update = try indent()
        case .unindent:
            update = try unindent()
        case .codeBlock:
            update = try codeBlock()
        case .quote:
            update = try quote()
        case .link:
            fatalError()
        }

        return update
    }

    /// Returns currently reversed (active) actions on the composer model.
    var reversedActions: Set<ComposerAction> {
        Set(actionStates().compactMap { (key: ComposerAction, value: ActionState) in
            value == .reversed ? key : nil
        })
    }
}
