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

public enum WysiwygAction {
    /// Apply bold formatting to the current selection.
    case bold
    /// Apply italic formatting to the current selection.
    case italic
    /// Apply strike through formatting to the current selection.
    case strikeThrough
    /// Apply underline formatting to the current selection.
    case underline
    /// Apply inline code formatting to the current selection
    case inlineCode
    /// Create a link at current selection
    case link(url: String)
    /// Undo last model operation.
    case undo
    /// Redo latest undone operation.
    case redo
    /// Create an ordered list.
    case orderedList
    /// Create an unordered list.
    case unorderedList
}
