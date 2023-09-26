//
// Copyright 2023 The Matrix.org Foundation C.I.C
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

extension ComposerAction {
    /// Returns `true` if action requires all current formatting to be re-applied on
    /// next character stroke when triggered on an empty selection.
    var requiresReapplyFormattingOnEmptySelection: Bool {
        switch self {
        case .bold, .italic, .strikeThrough, .underline, .inlineCode, .link, .undo, .redo:
            return false
        case .orderedList, .unorderedList, .indent, .unindent, .codeBlock, .quote:
            return true
        }
    }
}
