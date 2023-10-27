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
import UIKit

public protocol WysiwygComposerViewModelProtocol: AnyObject {
    /// The textView that the model manages.
    var textView: WysiwygTextView { get }

    /// Whether the current content of the composer is empty.
    var isContentEmpty: Bool { get }

    /// Update the composer compressed required height if it has changed.
    func updateCompressedHeightIfNeeded()

    /// Replace text in the model.
    ///
    /// - Parameters:
    ///   - range: Range to replace.
    ///   - replacementText: Replacement text to apply.
    func replaceText(range: NSRange, replacementText: String) -> Bool

    /// Select given range of text within the model.
    ///
    /// - Parameters:
    ///   - range: Range to select.
    func select(range: NSRange)

    /// Notify that the text view content has changed.
    func didUpdateText()

    /// Apply an enter/return key event.
    func enter()

    /// Get the ideal size for the composer's text view inside a SwiftUI context.
    ///
    /// - Parameter proposal: Proposed view size.
    /// - Returns: Ideal size for current context.
    @available(iOS 16.0, *)
    func getIdealSize(_ proposal: ProposedViewSize) -> CGSize
}
